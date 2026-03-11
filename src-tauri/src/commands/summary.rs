use crate::ai::summarizer::DailySummary;
use crate::commands::CmdResult;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type AppStateGuard<'a> = State<'a, Arc<Mutex<AppState>>>;

#[tauri::command]
pub async fn get_todays_summary(
    state: AppStateGuard<'_>,
) -> CmdResult<Option<DailySummary>> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.get_todays_summary().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_daily_summary(
    state: AppStateGuard<'_>,
) -> CmdResult<DailySummary> {
    let s = state.lock().await;

    // 1. Sync all connected integrations
    {
        let mut mgr = s.integrations.lock().await;
        let _ = mgr.sync_all().await;
    }

    // 2. Get unprocessed events
    let events = {
        let db = s.db.lock().await;
        db.get_unprocessed_events().map_err(|e| e.to_string())?
    };

    // 3. Generate AI summary
    let summary = s.ai
        .generate_daily_summary(&events)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Persist summary + mark events processed
    {
        let db = s.db.lock().await;
        db.save_summary(&summary).map_err(|e| e.to_string())?;
        let ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
        let _ = db.mark_events_processed(&ids);
    }

    Ok(summary)
}

#[tauri::command]
pub async fn get_summary_history(
    limit: Option<u32>,
    state: AppStateGuard<'_>,
) -> CmdResult<Vec<DailySummary>> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.get_summary_history(limit.unwrap_or(7)).map_err(|e| e.to_string())
}
