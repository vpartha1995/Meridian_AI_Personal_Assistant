use crate::integrations::plugin::{ConnectionStatus, IntegrationPlugin, UnifiedEvent};
use crate::integrations::{gchat, gmail, jira, outlook, slack, zoom};
use crate::storage::database::Database;
use crate::storage::keychain::Keychain;
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationInfo {
    pub id:     String,
    pub name:   String,
    pub icon:   String,
    pub status: ConnectionStatus,
}

pub struct IntegrationManager {
    plugins: Vec<Box<dyn IntegrationPlugin>>,
    db:      Arc<Mutex<Database>>,
}

impl IntegrationManager {
    pub fn new(db: Arc<Mutex<Database>>, keychain: Arc<Keychain>) -> Self {
        let plugins: Vec<Box<dyn IntegrationPlugin>> = vec![
            Box::new(gmail::GmailPlugin::new(Arc::clone(&keychain))),
            Box::new(outlook::OutlookPlugin::new(Arc::clone(&keychain))),
            Box::new(slack::SlackPlugin::new(Arc::clone(&keychain))),
            Box::new(zoom::ZoomPlugin::new(Arc::clone(&keychain))),
            Box::new(gchat::GChatPlugin::new(Arc::clone(&keychain))),
            Box::new(jira::JiraPlugin::new(Arc::clone(&keychain))),
        ];
        Self { plugins, db }
    }

    pub async fn list_integrations(&self) -> Vec<IntegrationInfo> {
        let mut out = Vec::new();
        for p in &self.plugins {
            let status = p.status().await;
            out.push(IntegrationInfo {
                id:     p.meta().id.to_string(),
                name:   p.meta().display_name.to_string(),
                icon:   p.meta().icon.to_string(),
                status,
            });
        }
        out
    }

    pub async fn authenticate(&self, id: &str) -> Result<()> {
        let plugin = self.find(id)?;
        plugin.authenticate().await.map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub async fn revoke(&self, id: &str) -> Result<()> {
        let plugin = self.find(id)?;
        plugin.revoke().await.map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub async fn status(&self, id: &str) -> Result<ConnectionStatus> {
        let plugin = self.find(id)?;
        Ok(plugin.status().await)
    }

    /// Sync all connected integrations and persist events to DB.
    pub async fn sync_all(&mut self) -> Result<Vec<UnifiedEvent>> {
        let since = Utc::now() - Duration::hours(24);
        let mut all_events = Vec::new();

        for plugin in &self.plugins {
            let s = plugin.status().await;
            if !matches!(s, ConnectionStatus::Connected { .. }) {
                continue;
            }
            match plugin.fetch_since(since).await {
                Ok(events) => {
                    let db = self.db.lock().await;
                    for ev in &events {
                        let _ = db.upsert_event(ev);
                    }
                    all_events.extend(events);
                }
                Err(e) => {
                    log::error!("Sync error for {}: {e}", plugin.meta().id);
                }
            }
        }
        Ok(all_events)
    }

    pub async fn sync_one(&mut self, id: &str) -> Result<Vec<UnifiedEvent>> {
        let since = Utc::now() - Duration::hours(24);
        let plugin = self.find(id)?;
        let events = plugin
            .fetch_since(since)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let db = self.db.lock().await;
        for ev in &events {
            let _ = db.upsert_event(ev);
        }
        Ok(events)
    }

    fn find(&self, id: &str) -> Result<&dyn IntegrationPlugin> {
        self.plugins
            .iter()
            .find(|p| p.meta().id == id)
            .map(|p| p.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Integration '{id}' not found"))
    }
}
