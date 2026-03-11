use crate::integrations::plugin::*;
use crate::storage::keychain::Keychain;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const ID: &str = "gmail";

pub struct GmailPlugin {
    client:   Client,
    keychain: Arc<Keychain>,
    meta:     PluginMeta,
}

impl GmailPlugin {
    pub fn new(keychain: Arc<Keychain>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            keychain,
            meta: PluginMeta {
                id:           ID,
                display_name: "Gmail",
                icon:         "mail",
                description:  "Google Gmail — emails and threads",
            },
        }
    }
}

#[async_trait]
impl IntegrationPlugin for GmailPlugin {
    fn meta(&self) -> &PluginMeta { &self.meta }

    async fn authenticate(&self) -> Result<(), IntegrationError> {
        // PKCE OAuth2 flow
        use oauth2::{
            basic::BasicClient, AuthUrl, ClientId, CsrfToken, PkceCodeChallenge,
            RedirectUrl, TokenUrl,
        };
        use std::net::TcpListener;

        // Read client ID from settings or use placeholder
        let client_id = std::env::var("GMAIL_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_GMAIL_CLIENT_ID.apps.googleusercontent.com".to_string());

        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://127.0.0.1:{port}/callback");

        let oauth_client = BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| IntegrationError::OAuth(e.to_string()))?,
            Some(
                TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .map_err(|e| IntegrationError::OAuth(e.to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.clone())
                .map_err(|e| IntegrationError::OAuth(e.to_string()))?,
        );

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _csrf) = oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(oauth2::Scope::new(
                "https://www.googleapis.com/auth/gmail.readonly".to_string(),
            ))
            .add_scope(oauth2::Scope::new(
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
            ))
            .set_pkce_challenge(pkce_challenge)
            .url();

        // Open browser
        open::that(auth_url.as_str())
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        // Wait for redirect on localhost
        let code = wait_for_oauth_code(port)
            .await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        // Exchange code for token
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
            "expires_at":    token.expires_in().map(|d| {
                (Utc::now() + chrono::Duration::seconds(d.as_secs() as i64)).to_rfc3339()
            }),
        });

        self.keychain
            .store_token(ID, &token_json.to_string())
            .map_err(IntegrationError::Storage)?;

        // Fetch and store account email
        if let Ok(email) = self.fetch_email(token.access_token().secret()).await {
            let _ = self.keychain.store_account(ID, &email);
        }

        Ok(())
    }

    async fn fetch_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnifiedEvent>, IntegrationError> {
        let token_json = self.keychain.get_token(ID).map_err(|_| IntegrationError::NotAuthenticated)?;
        let token: serde_json::Value = serde_json::from_str(&token_json)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let access_token = token["access_token"]
            .as_str()
            .ok_or(IntegrationError::NotAuthenticated)?;

        let after = since.timestamp();
        let resp: GmailListResp = self
            .client
            .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
            .bearer_auth(access_token)
            .query(&[("q", &format!("after:{after}")), ("maxResults", &"50".to_string())])
            .send()
            .await?
            .json()
            .await?;

        let mut events = Vec::new();
        for msg_ref in resp.messages.unwrap_or_default().iter().take(30) {
            if let Ok(msg) = self.fetch_message(access_token, &msg_ref.id).await {
                events.push(msg);
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

impl GmailPlugin {
    async fn fetch_email(&self, token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct UserInfo { email: String }
        let info: UserInfo = self
            .client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;
        Ok(info.email)
    }

    async fn fetch_message(&self, token: &str, id: &str) -> Result<UnifiedEvent> {
        let msg: GmailMsg = self
            .client
            .get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{id}"))
            .bearer_auth(token)
            .query(&[("format", "metadata"),("metadataHeaders","Subject"),("metadataHeaders","From")])
            .send()
            .await?
            .json()
            .await?;

        let subject = hdr(&msg, "Subject").unwrap_or_else(|| "(no subject)".to_string());
        let from    = hdr(&msg, "From").unwrap_or_else(|| "Unknown".to_string());

        Ok(UnifiedEvent {
            id:          format!("gmail_{}", msg.id),
            source:      ID.to_string(),
            title:       subject,
            body:        msg.snippet.unwrap_or_default(),
            sender:      Some(from),
            url:         Some(format!("https://mail.google.com/mail/u/0/#inbox/{}", msg.id)),
            received_at: Utc::now(),
            priority:    None,
        })
    }
}

fn hdr(msg: &GmailMsg, name: &str) -> Option<String> {
    msg.payload.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.clone())
}

/// Public re-export for other integration modules
pub async fn wait_for_oauth_code_pub(port: u16) -> anyhow::Result<String> {
    wait_for_oauth_code(port).await
}

async fn wait_for_oauth_code(port: u16) -> anyhow::Result<String> {
    use tiny_http::{Response, Server};
    let server = Server::http(format!("127.0.0.1:{port}"))
        .map_err(|e| anyhow::anyhow!("HTTP server: {e}"))?;

    let request = tokio::task::spawn_blocking(move || {
        server.recv().map_err(|e| anyhow::anyhow!("Recv: {e}"))
    })
    .await??;

    let url = format!("http://localhost{}", request.url());
    let parsed = url::Url::parse(&url)?;
    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| anyhow::anyhow!("No code in callback"))?;

    let html = "<html><body><h2>Meridian connected!</h2><p>You can close this tab.</p></body></html>";
    let _ = request.respond(Response::from_string(html));
    Ok(code)
}

#[derive(Deserialize)]
struct GmailListResp { messages: Option<Vec<MsgRef>> }
#[derive(Deserialize)]
struct MsgRef { id: String }
#[derive(Deserialize, Serialize)]
struct GmailMsg {
    id:      String,
    snippet: Option<String>,
    payload: GmailPayload,
}
#[derive(Deserialize, Serialize)]
struct GmailPayload { headers: Vec<Header> }
#[derive(Deserialize, Serialize)]
struct Header { name: String, value: String }
