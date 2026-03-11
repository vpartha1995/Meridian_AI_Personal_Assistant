use crate::integrations::plugin::*;
use crate::storage::keychain::Keychain;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

const ID: &str = "zoom";

pub struct ZoomPlugin {
    client:   Client,
    keychain: Arc<Keychain>,
    meta:     PluginMeta,
}

impl ZoomPlugin {
    pub fn new(keychain: Arc<Keychain>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            keychain,
            meta: PluginMeta {
                id:           ID,
                display_name: "Zoom",
                icon:         "video",
                description:  "Zoom — meetings and recordings",
            },
        }
    }
}

#[async_trait]
impl IntegrationPlugin for ZoomPlugin {
    fn meta(&self) -> &PluginMeta { &self.meta }

    async fn authenticate(&self) -> Result<(), IntegrationError> {
        use std::net::TcpListener;

        let client_id = std::env::var("ZOOM_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_ZOOM_CLIENT_ID".to_string());

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://localhost:{port}/callback");
        let auth_url = format!(
            "https://zoom.us/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={redirect_uri}"
        );

        open::that(&auth_url)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let code = super::gmail::wait_for_oauth_code_pub(port)
            .await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let client_secret = std::env::var("ZOOM_CLIENT_SECRET")
            .unwrap_or_else(|_| "YOUR_ZOOM_CLIENT_SECRET".to_string());
        let creds = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{client_id}:{client_secret}"),
        );

        let resp: serde_json::Value = self
            .client
            .post("https://zoom.us/oauth/token")
            .header("Authorization", format!("Basic {creds}"))
            .query(&[("grant_type", "authorization_code"), ("code", &code), ("redirect_uri", &redirect_uri)])
            .send()
            .await?
            .json()
            .await?;

        self.keychain.store_token(ID, &resp.to_string()).map_err(IntegrationError::Storage)?;

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

        let from = since.format("%Y-%m-%d").to_string();
        let to   = Utc::now().format("%Y-%m-%d").to_string();

        let resp: ZoomMeetingsResp = self
            .client
            .get("https://api.zoom.us/v2/users/me/meetings")
            .bearer_auth(access_token)
            .query(&[("type", "scheduled"), ("from", &from), ("to", &to), ("page_size", "20")])
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.meetings.unwrap_or_default().into_iter().map(|m| UnifiedEvent {
            id:          format!("zoom_{}", m.id),
            source:      ID.to_string(),
            title:       m.topic.clone(),
            body:        format!("Scheduled: {} | Duration: {} min", m.start_time.as_deref().unwrap_or("TBD"), m.duration.unwrap_or(0)),
            sender:      None,
            url:         m.join_url.clone(),
            received_at: m.start_time.as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(Utc::now),
            priority:    Some("medium".to_string()),
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

impl ZoomPlugin {
    async fn fetch_email(&self, token: &str) -> anyhow::Result<String> {
        let me: serde_json::Value = self.client
            .get("https://api.zoom.us/v2/users/me")
            .bearer_auth(token).send().await?.json().await?;
        Ok(me["email"].as_str().unwrap_or_default().to_string())
    }
}

#[derive(Deserialize)]
struct ZoomMeetingsResp { meetings: Option<Vec<ZoomMeeting>> }
#[derive(Deserialize)]
struct ZoomMeeting {
    id:         serde_json::Value,
    topic:      String,
    start_time: Option<String>,
    duration:   Option<u32>,
    join_url:   Option<String>,
}
