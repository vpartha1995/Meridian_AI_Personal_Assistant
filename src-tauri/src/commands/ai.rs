use crate::ai::ollama::pull_model_with_progress;
use crate::commands::CmdResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

type AppStateGuard<'a> = State<'a, Arc<Mutex<AppState>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub available:    bool,
    pub model_ready:  bool,
    pub model_name:   String,
}

#[tauri::command]
pub async fn check_ollama_status(
    state: AppStateGuard<'_>,
) -> CmdResult<OllamaStatus> {
    let s = state.lock().await;
    let available   = s.ai.is_available().await;
    let model_ready = if available { s.ai.model_ready().await } else { false };
    Ok(OllamaStatus {
        available,
        model_ready,
        model_name: "phi3:mini".to_string(),
    })
}

/// Pull an Ollama model with real-time progress events.
/// Emits `ollama://pull-progress` events: `{ model, status, total, completed, percent }`.
#[tauri::command]
pub async fn pull_model(
    model: Option<String>,
    app:   AppHandle,
) -> CmdResult<()> {
    let model_name = model.as_deref().unwrap_or("phi3:mini");
    pull_model_with_progress(&app, model_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn draft_email(
    context:      String,
    instructions: String,
    state: AppStateGuard<'_>,
) -> CmdResult<String> {
    let s = state.lock().await;
    s.ai.draft_email(&context, &instructions).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rephrase_text(
    text:  String,
    tone:  String,
    state: AppStateGuard<'_>,
) -> CmdResult<String> {
    let s = state.lock().await;
    s.ai.rephrase_text(&text, &tone).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ask_assistant(
    question: String,
    context:  String,
    state: AppStateGuard<'_>,
) -> CmdResult<String> {
    let s = state.lock().await;
    s.ai.ask(&question, &context).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn summarize_text(
    text:  String,
    state: AppStateGuard<'_>,
) -> CmdResult<String> {
    let s = state.lock().await;
    s.ai.summarize_text(&text).await.map_err(|e| e.to_string())
}
