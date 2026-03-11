use crate::commands::CmdResult;
use crate::integrations::manager::IntegrationInfo;
use crate::integrations::plugin::ConnectionStatus;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type AppStateGuard<'a> = State<'a, Arc<Mutex<AppState>>>;

#[tauri::command]
pub async fn list_integrations(
    state: AppStateGuard<'_>,
) -> CmdResult<Vec<IntegrationInfo>> {
    let s = state.lock().await;
    let mgr = s.integrations.lock().await;
    Ok(mgr.list_integrations().await)
}

#[tauri::command]
pub async fn connect_integration(
    id:    String,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s = state.lock().await;
    let mgr = s.integrations.lock().await;
    mgr.authenticate(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_integration(
    id:    String,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s = state.lock().await;
    let mgr = s.integrations.lock().await;
    mgr.revoke(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_integration_status(
    id:    String,
    state: AppStateGuard<'_>,
) -> CmdResult<ConnectionStatus> {
    let s = state.lock().await;
    let mgr = s.integrations.lock().await;
    mgr.status(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trigger_sync(
    id:    Option<String>,
    state: AppStateGuard<'_>,
) -> CmdResult<usize> {
    let s = state.lock().await;
    let mut mgr = s.integrations.lock().await;
    let events = match id {
        Some(ref id) => mgr.sync_one(id).await.map_err(|e| e.to_string())?,
        None         => mgr.sync_all().await.map_err(|e| e.to_string())?,
    };
    Ok(events.len())
}
