use crate::integrations::plugin::*;
use crate::storage::keychain::Keychain;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const ID: &str = "slack";

pub struct SlackPlugin {
    client:   Client,
    keychain: Arc<Keychain>,
    meta:     PluginMeta,
}

impl SlackPlugin {
    pub fn new(keychain: Arc<Keychain>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            keychain,
            meta: PluginMeta {
                id:           ID,
                display_name: "Slack",
                icon:         "message-square",
                description:  "Slack — channels, DMs and mentions",
            },
        }
    }

    fn token(&self) -> Result<String, IntegrationError> {
        let json = self.keychain.get_token(ID).map_err(|_| IntegrationError::NotAuthenticated)?;
        let v: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        v["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(IntegrationError::NotAuthenticated)
    }
}

#[async_trait]
impl IntegrationPlugin for SlackPlugin {
    fn meta(&self) -> &PluginMeta { &self.meta }

    async fn authenticate(&self) -> Result<(), IntegrationError> {
        use std::net::TcpListener;

        let (client_id, client_secret) = self.keychain
            .get_oauth_creds(ID)
            .map_err(|_| IntegrationError::AuthFailed(
                "Slack OAuth credentials are not configured. \
                 Go to Settings → Integrations → Slack Setup and enter \
                 your Slack App Client ID and Client Secret.".to_string()
            ))?;

        if client_id.is_empty() || client_secret.is_empty() {
            return Err(IntegrationError::AuthFailed(
                "Slack Client ID or Secret is empty. \
                 Go to Settings → Integrations → Slack Setup and enter \
                 your credentials from api.slack.com/apps.".to_string()
            ));
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://localhost:{port}/callback");
        let scopes = "channels:history,channels:read,groups:history,groups:read,im:history,im:read,users:read,search:read";
        let auth_url = format!(
            "https://slack.com/oauth/v2/authorize?client_id={client_id}&scope={scopes}&redirect_uri={redirect_uri}"
        );

        open::that(&auth_url)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let code = super::gmail::wait_for_oauth_code_pub(port)
            .await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        // Exchange code
        let resp: SlackOAuthResp = self
            .client
            .post("https://slack.com/api/oauth.v2.access")
            .form(&[
                ("code", code.as_str()),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("redirect_uri", &redirect_uri),
            ])
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            return Err(IntegrationError::AuthFailed(
                resp.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let token_json = serde_json::json!({
            "access_token": resp.access_token,
            "team": resp.team.as_ref().map(|t| &t.name),
        });
        self.keychain
            .store_token(ID, &token_json.to_string())
            .map_err(IntegrationError::Storage)?;

        let workspace = resp.team.map(|t| t.name).unwrap_or_else(|| "Slack".to_string());
        let _ = self.keychain.store_account(ID, &workspace);

        Ok(())
    }

    async fn fetch_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnifiedEvent>, IntegrationError> {
        let token  = self.token()?;
        let oldest = since.timestamp().to_string();

        // Search for messages mentioning the user
        let resp: SlackSearchResp = self
            .client
            .get("https://slack.com/api/search.messages")
            .bearer_auth(&token)
            .query(&[
                ("query", "has:mention"),
                ("count", "20"),
                ("sort", "timestamp"),
            ])
            .send()
            .await?
            .json()
            .await?;

        let mut events = Vec::new();
        if let Some(msgs) = resp.messages {
            for m in msgs.matches.unwrap_or_default() {
                if m.ts.parse::<f64>().unwrap_or(0.0) < since.timestamp() as f64 {
                    continue;
                }
                events.push(UnifiedEvent {
                    id:          format!("slack_{}", m.ts),
                    source:      ID.to_string(),
                    title:       format!("#{} — {}", m.channel.name, &m.text[..m.text.len().min(60)]),
                    body:        m.text.clone(),
                    sender:      m.username.clone(),
                    url:         m.permalink.clone(),
                    received_at: chrono::DateTime::from_timestamp(
                        m.ts.parse::<f64>().unwrap_or(0.0) as i64, 0,
                    ).unwrap_or_else(Utc::now),
                    priority:    None,
                });
            }
        }

        let _ = oldest; // suppress warning
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
struct SlackOAuthResp {
    ok:           bool,
    error:        Option<String>,
    access_token: Option<String>,
    team:         Option<SlackTeam>,
}
#[derive(Deserialize)]
struct SlackTeam { name: String }
#[derive(Deserialize)]
struct SlackSearchResp { messages: Option<SlackMessages> }
#[derive(Deserialize)]
struct SlackMessages { matches: Option<Vec<SlackMsg>> }
#[derive(Deserialize)]
struct SlackMsg {
    ts:        String,
    text:      String,
    username:  Option<String>,
    permalink: Option<String>,
    channel:   SlackChannel,
}
#[derive(Deserialize)]
struct SlackChannel { name: String }
