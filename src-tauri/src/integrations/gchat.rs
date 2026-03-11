use crate::integrations::plugin::*;
use crate::storage::keychain::Keychain;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

const ID: &str = "gchat";

pub struct GChatPlugin {
    client:   Client,
    keychain: Arc<Keychain>,
    meta:     PluginMeta,
}

impl GChatPlugin {
    pub fn new(keychain: Arc<Keychain>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            keychain,
            meta: PluginMeta {
                id:           ID,
                display_name: "Google Chat",
                icon:         "message-circle",
                description:  "Google Chat — spaces and direct messages",
            },
        }
    }
}

#[async_trait]
impl IntegrationPlugin for GChatPlugin {
    fn meta(&self) -> &PluginMeta { &self.meta }

    async fn authenticate(&self) -> Result<(), IntegrationError> {
        use oauth2::{basic::BasicClient, AuthUrl, ClientId, CsrfToken,
                     PkceCodeChallenge, RedirectUrl, TokenUrl};
        use std::net::TcpListener;

        let client_id = std::env::var("GCHAT_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_GCHAT_CLIENT_ID.apps.googleusercontent.com".to_string());

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://localhost:{port}/callback");
        let oauth_client = BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| IntegrationError::OAuth(e.to_string()))?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .map_err(|e| IntegrationError::OAuth(e.to_string()))?),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri).map_err(|e| IntegrationError::OAuth(e.to_string()))?,
        );

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _) = oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("https://www.googleapis.com/auth/chat.messages.readonly".to_string()))
            .add_scope(oauth2::Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        open::that(auth_url.as_str())
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let code = super::gmail::wait_for_oauth_code_pub(port)
            .await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        use oauth2::{reqwest::async_http_client, AuthorizationCode, TokenResponse};
        let token = oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| IntegrationError::OAuth(e.to_string()))?;

        let token_json = serde_json::json!({
            "access_token":  token.access_token().secret(),
            "refresh_token": token.refresh_token().map(|t| t.secret()),
        });
        self.keychain.store_token(ID, &token_json.to_string()).map_err(IntegrationError::Storage)?;

        // Store account
        #[derive(Deserialize)]
        struct UserInfo { email: String }
        if let Ok(resp) = self.client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(token.access_token().secret())
            .send()
            .await
        {
            if let Ok(info) = resp.json::<UserInfo>().await {
                let _ = self.keychain.store_account(ID, &info.email);
            }
        }

        Ok(())
    }

    async fn fetch_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnifiedEvent>, IntegrationError> {
        let token_json = self.keychain.get_token(ID)
            .map_err(|_| IntegrationError::NotAuthenticated)?;
        let token: serde_json::Value = serde_json::from_str(&token_json)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let access_token = token["access_token"].as_str()
            .ok_or(IntegrationError::NotAuthenticated)?;

        // List spaces
        let spaces_resp: GChatSpacesResp = self
            .client
            .get("https://chat.googleapis.com/v1/spaces")
            .bearer_auth(access_token)
            .send()
            .await?
            .json()
            .await?;

        let mut events = Vec::new();
        for space in spaces_resp.spaces.unwrap_or_default().iter().take(5) {
            let filter = format!("createTime > \"{}\"", since.to_rfc3339());
            if let Ok(msgs_resp) = self.client
                .get(format!("https://chat.googleapis.com/v1/{}/messages", space.name))
                .bearer_auth(access_token)
                .query(&[("pageSize", "20"), ("filter", &filter)])
                .send()
                .await
            {
                if let Ok(msgs) = msgs_resp.json::<GChatMsgsResp>().await {
                    for msg in msgs.messages.unwrap_or_default() {
                        events.push(UnifiedEvent {
                            id:          format!("gchat_{}", msg.name.replace('/', "_")),
                            source:      ID.to_string(),
                            title:       format!("{} — {}", space.display_name.as_deref().unwrap_or("Chat"), &msg.text[..msg.text.len().min(60)]),
                            body:        msg.text.clone(),
                            sender:      msg.sender.as_ref().map(|s| s.display_name.clone()),
                            url:         None,
                            received_at: msg.create_time.as_deref()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or_else(Utc::now),
                            priority:    None,
                        });
                    }
                }
            }
        }
        Ok(events)
    }

    async fn revoke(&self) -> Result<(), IntegrationError> {
        let _ = self.keychain.delete_token(ID);
        Ok(())
    }

    async fn status(&self) -> ConnectionStatus {
        match self.keychain.get_account(ID) {
            Ok(account) => ConnectionStatus::Connected { account },
            Err(_) => ConnectionStatus::Disconnected,
        }
    }
}

#[derive(Deserialize)]
struct GChatSpacesResp { spaces: Option<Vec<GChatSpace>> }
#[derive(Deserialize)]
struct GChatSpace { name: String, #[serde(rename="displayName")] display_name: Option<String> }
#[derive(Deserialize)]
struct GChatMsgsResp { messages: Option<Vec<GChatMsg>> }
#[derive(Deserialize)]
struct GChatMsg {
    name:        String,
    text:        String,
    sender:      Option<GChatSender>,
    #[serde(rename="createTime")]
    create_time: Option<String>,
}
#[derive(Deserialize)]
struct GChatSender { #[serde(rename="displayName")] display_name: String }
