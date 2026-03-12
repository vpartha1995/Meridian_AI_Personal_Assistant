use crate::commands::CmdResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type AppStateGuard<'a> = State<'a, Arc<Mutex<AppState>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme:               String,
    pub scan_interval_mins:  u32,
    pub working_hours_start: u8,
    pub working_hours_end:   u8,
    pub health_nudges:       bool,
    pub notifications:       bool,
    pub news_topics:         Vec<String>,
    pub user_name:           String,
    pub avatar_url:          Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme:               "dark".to_string(),
            scan_interval_mins:  60,
            working_hours_start: 9,
            working_hours_end:   18,
            health_nudges:       true,
            notifications:       true,
            news_topics:         vec!["technology".to_string(), "ai".to_string()],
            user_name:           "User".to_string(),
            avatar_url:          None,
        }
    }
}

#[tauri::command]
pub async fn get_settings(
    state: AppStateGuard<'_>,
) -> CmdResult<AppSettings> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    match db.get_setting("app_settings").map_err(|e| e.to_string())? {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        None       => Ok(AppSettings::default()),
    }
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state:    AppStateGuard<'_>,
) -> CmdResult<()> {
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    let s    = state.lock().await;
    let db   = s.db.lock().await;
    db.set_setting("app_settings", &json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_theme(
    state: AppStateGuard<'_>,
) -> CmdResult<String> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    Ok(db.get_setting("theme").map_err(|e| e.to_string())?.unwrap_or_else(|| "dark".to_string()))
}

#[tauri::command]
pub async fn set_theme(
    theme: String,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.set_setting("theme", &theme).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn log_health_action(
    action: String,
    state:  AppStateGuard<'_>,
) -> CmdResult<()> {
    let key = format!("last_{action}");
    let s   = state.lock().await;
    let db  = s.db.lock().await;
    db.set_setting(&key, &chrono::Utc::now().to_rfc3339()).map_err(|e| e.to_string())
}

/// Store the OAuth app credentials (client_id + client_secret) for an
/// integration in the OS keychain.  The secret is never returned to the
/// frontend — only the client_id is exposed via `get_oauth_client_id`.
#[tauri::command]
pub async fn set_oauth_creds(
    integration_id: String,
    client_id:      String,
    client_secret:  String,
    state:          AppStateGuard<'_>,
) -> CmdResult<()> {
    let s = state.lock().await;
    s.keychain
        .store_oauth_creds(&integration_id, &client_id, &client_secret)
        .map_err(|e| e.to_string())
}

/// Returns the stored client_id for `integration_id`, or `null` if not
/// configured.  The client_secret is intentionally not returned.
#[tauri::command]
pub async fn get_oauth_client_id(
    integration_id: String,
    state:          AppStateGuard<'_>,
) -> CmdResult<Option<String>> {
    let s = state.lock().await;
    Ok(s.keychain.get_oauth_client_id(&integration_id))
}

/// Remove stored OAuth app credentials for an integration.
#[tauri::command]
pub async fn delete_oauth_creds(
    integration_id: String,
    state:          AppStateGuard<'_>,
) -> CmdResult<()> {
    let s = state.lock().await;
    let _ = s.keychain.delete_oauth_creds(&integration_id);
    Ok(())
}
