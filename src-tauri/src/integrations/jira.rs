use crate::integrations::plugin::*;
use crate::storage::keychain::Keychain;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const ID: &str = "jira";

pub struct JiraPlugin {
    client:   Client,
    keychain: Arc<Keychain>,
    meta:     PluginMeta,
}

impl JiraPlugin {
    pub fn new(keychain: Arc<Keychain>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            keychain,
            meta: PluginMeta {
                id:           ID,
                display_name: "Jira",
                icon:         "ticket",
                description:  "Jira — tickets, assignments and project updates",
            },
        }
    }
}

#[async_trait]
impl IntegrationPlugin for JiraPlugin {
    fn meta(&self) -> &PluginMeta { &self.meta }

    async fn authenticate(&self) -> Result<(), IntegrationError> {
        use std::net::TcpListener;

        let client_id = std::env::var("JIRA_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_JIRA_CLIENT_ID".to_string());

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://localhost:{port}/callback");
        let auth_url = format!(
            "https://auth.atlassian.com/authorize?audience=api.atlassian.com\
            &client_id={client_id}&scope=read%3Ajira-work%20read%3Ajira-user\
            &redirect_uri={redirect_uri}&state=meridian&response_type=code\
            &prompt=consent"
        );

        open::that(&auth_url)
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let code = super::gmail::wait_for_oauth_code_pub(port)
            .await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;

        let client_secret = std::env::var("JIRA_CLIENT_SECRET")
            .unwrap_or_else(|_| "YOUR_JIRA_CLIENT_SECRET".to_string());

        let resp: serde_json::Value = self
            .client
            .post("https://auth.atlassian.com/oauth/token")
            .json(&serde_json::json!({
                "grant_type":    "authorization_code",
                "client_id":     client_id,
                "client_secret": client_secret,
                "code":          code,
                "redirect_uri":  redirect_uri,
            }))
            .send()
            .await?
            .json()
            .await?;

        self.keychain
            .store_token(ID, &resp.to_string())
            .map_err(IntegrationError::Storage)?;

        // Fetch accessible resources (cloud IDs)
        let token = resp["access_token"].as_str().unwrap_or_default().to_string();
        if let Ok(sites) = self.fetch_sites(&token).await {
            if let Some(site) = sites.first() {
                let _ = self.keychain.store_account(ID, &site.url);
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

        let sites = self.fetch_sites(access_token).await
            .map_err(|e| IntegrationError::AuthFailed(e.to_string()))?;
        let cloud_id = sites.first()
            .map(|s| s.id.clone())
            .ok_or_else(|| IntegrationError::AuthFailed("No Jira sites accessible".to_string()))?;

        let updated = since.format("%Y-%m-%d %H:%M").to_string();
        let jql = format!("assignee = currentUser() AND updated >= \"{updated}\" ORDER BY updated DESC");

        let resp: JiraSearchResp = self
            .client
            .get(format!(
                "https://api.atlassian.com/ex/jira/{cloud_id}/rest/api/3/search"
            ))
            .bearer_auth(access_token)
            .query(&[("jql", &jql), ("maxResults", &"30".to_string()),
                     ("fields", &"summary,status,priority,assignee".to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.issues.into_iter().map(|issue| {
            let site_url = self.keychain.get_account(ID).unwrap_or_default();
            UnifiedEvent {
                id:          format!("jira_{}", issue.id),
                source:      ID.to_string(),
                title:       format!("[{}] {}", issue.key, issue.fields.summary),
                body:        format!(
                    "Status: {} | Priority: {}",
                    issue.fields.status.name,
                    issue.fields.priority.as_ref().map(|p| p.name.as_str()).unwrap_or("Medium"),
                ),
                sender:      None,
                url:         Some(format!("{site_url}/browse/{}", issue.key)),
                received_at: Utc::now(),
                priority:    Some(match issue.fields.priority.as_ref().map(|p| p.name.as_str()) {
                    Some("Highest") | Some("High") => "high".to_string(),
                    Some("Low") | Some("Lowest")   => "low".to_string(),
                    _ => "medium".to_string(),
                }),
            }
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

impl JiraPlugin {
    async fn fetch_sites(&self, token: &str) -> anyhow::Result<Vec<JiraSite>> {
        let sites: Vec<JiraSite> = self
            .client
            .get("https://api.atlassian.com/oauth/token/accessible-resources")
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;
        Ok(sites)
    }
}

#[derive(Deserialize)]
struct JiraSite { id: String, url: String }
#[derive(Deserialize)]
struct JiraSearchResp { issues: Vec<JiraIssue> }
#[derive(Deserialize)]
struct JiraIssue { id: String, key: String, fields: JiraFields }
#[derive(Deserialize)]
struct JiraFields {
    summary:  String,
    status:   JiraStatus,
    priority: Option<JiraPriority>,
}
#[derive(Deserialize)]
struct JiraStatus   { name: String }
#[derive(Deserialize)]
struct JiraPriority { name: String }
