use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Shared data model ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEvent {
    pub id:          String,
    pub source:      String,
    pub title:       String,
    pub body:        String,
    pub sender:      Option<String>,
    pub url:         Option<String>,
    pub received_at: DateTime<Utc>,
    pub priority:    Option<String>,
}

// ── Plugin metadata ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub id:           &'static str,
    pub display_name: &'static str,
    pub icon:         &'static str,
    pub description:  &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum ConnectionStatus {
    #[serde(rename = "connected")]
    Connected { account: String },
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "error")]
    Error { message: String },
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Not authenticated — connect the integration first")]
    NotAuthenticated,
    #[error("Auth failed: {0}")]
    AuthFailed(String),
    #[error("API error {status}: {message}")]
    ApiError { status: u16, message: String },
    #[error("Token expired")]
    TokenExpired,
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Storage error: {0}")]
    Storage(#[from] anyhow::Error),
    #[error("OAuth error: {0}")]
    OAuth(String),
}

// ── Plugin trait ──────────────────────────────────────────────────────────────

#[async_trait]
pub trait IntegrationPlugin: Send + Sync {
    fn meta(&self) -> &PluginMeta;

    /// Open OAuth consent in system browser and complete PKCE flow.
    async fn authenticate(&self) -> Result<(), IntegrationError>;

    /// Fetch new events since `since` (delta sync).
    async fn fetch_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnifiedEvent>, IntegrationError>;

    /// Remove stored tokens and disconnect.
    async fn revoke(&self) -> Result<(), IntegrationError>;

    /// Current connection state.
    async fn status(&self) -> ConnectionStatus;
}
