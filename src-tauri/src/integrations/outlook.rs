use crate::integrations::plugin::*;
use crate::storage::keychain::Keychain;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

const ID: &str = "outlook";

pub struct OutlookPlugin {
    client:   Client,
    keychain: Arc<Keychain>,
    meta:     PluginMeta,
}

impl OutlookPlugin {
    pub fn new(keychain: Arc<Keychain>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            keychain,
            meta: PluginMeta {
                id:           ID,
                display_name: "Outlook",
                icon:         "inbox",
                description:  "Microsoft Outlook / Exchange email",
            },
        }
    }
}

#[async_trait]
impl IntegrationPlugin for OutlookPlugin {
    fn meta(&self) -> &PluginMeta { &self.meta }

    async fn authenticate(&self) -> Result<(), IntegrationError> {
        use std::net::TcpListener;

        let tenant_id = "common";
        let client_id = std::env::var("OUTLOOK_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_AZURE_APP_CLIENT_ID".to_string());

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://localhost:{port}/callback");
        let scopes = "Mail.Read User.Read offline_access";
        let auth_url = format!(
            "https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/authorize\
            ?client_id={client_id}&response_type=code\
            &redirect_uri={redirect_uri}\
            &response_mode=query&scope={scopes}"
        );

        open::that(&auth_url)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let code = super::gmail::wait_for_oauth_code_pub(port)
            .await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let resp: serde_json::Value = self
            .client
            .post(format!(
                "https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token"
            ))
            .form(&[
                ("grant_type",   "authorization_code"),
                ("client_id",    &client_id),
                ("code",         &code),
                ("redirect_uri", &redirect_uri),
                ("scope",        scopes),
            ])
            .send()
            .await?
            .json()
            .await?;

        self.keychain.store_token(ID, &resp.to_string()).map_err(IntegrationError::Storage)?;

        // Fetch account email
        if let Some(token) = resp["access_token"].as_str() {
            if let Ok(email) = self.fetch_email(token).await {
                let _ = self.keychain.store_account(ID, &email);
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

        let filter = format!(
            "receivedDateTime ge {}",
            since.format("%Y-%m-%dT%H:%M:%SZ")
        );

        let resp: MsGraphMailResp = self
            .client
            .get("https://graph.microsoft.com/v1.0/me/messages")
            .bearer_auth(access_token)
            .query(&[
                ("$filter",  &filter),
                ("$top",     &"50".to_string()),
                ("$select",  &"id,subject,from,bodyPreview,webLink,receivedDateTime,importance".to_string()),
                ("$orderby", &"receivedDateTime desc".to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.value.into_iter().map(|m| UnifiedEvent {
            id:          format!("outlook_{}", m.id),
            source:      ID.to_string(),
            title:       m.subject.unwrap_or_else(|| "(no subject)".to_string()),
            body:        m.body_preview.unwrap_or_default(),
            sender:      m.from.as_ref().and_then(|f| f.email_address.as_ref().map(|e| e.address.clone())),
            url:         m.web_link,
            received_at: m.received_date_time
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(Utc::now),
            priority:    Some(match m.importance.as_deref() {
                Some("high")   => "high",
                Some("normal") => "medium",
                _              => "low",
            }.to_string()),
        }).collect())
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

impl OutlookPlugin {
    async fn fetch_email(&self, token: &str) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Me { mail: Option<String>, #[serde(rename="userPrincipalName")] upn: Option<String> }
        let me: Me = self.client.get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(token).send().await?.json().await?;
        Ok(me.mail.or(me.upn).unwrap_or_default())
    }
}

#[derive(Deserialize)]
struct MsGraphMailResp { value: Vec<MsMsg> }
#[derive(Deserialize)]
struct MsMsg {
    id:                   String,
    subject:              Option<String>,
    #[serde(rename="bodyPreview")]
    body_preview:         Option<String>,
    from:                 Option<MsSender>,
    #[serde(rename="webLink")]
    web_link:             Option<String>,
    #[serde(rename="receivedDateTime")]
    received_date_time:   Option<String>,
    importance:           Option<String>,
}
#[derive(Deserialize)]
struct MsSender { #[serde(rename="emailAddress")] email_address: Option<MsEmail> }
#[derive(Deserialize)]
struct MsEmail { address: String }
