use crate::commands::CmdResult;
use crate::state::AppState;
use crate::tasks::manager::{NewReminder, NewTask, Reminder, Task};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type AppStateGuard<'a> = State<'a, Arc<Mutex<AppState>>>;

#[tauri::command]
pub async fn list_tasks(
    include_completed: Option<bool>,
    state: AppStateGuard<'_>,
) -> CmdResult<Vec<Task>> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.list_tasks(include_completed.unwrap_or(false)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_task(
    req:   NewTask,
    state: AppStateGuard<'_>,
) -> CmdResult<Task> {
    let task = Task::new(req);
    let s    = state.lock().await;
    let db   = s.db.lock().await;
    db.insert_task(&task).map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
pub async fn update_task(
    task:  Task,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.update_task(&task).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(
    id:    String,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.delete_task(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_task(
    id:    String,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    let mut tasks = db.list_tasks(false).map_err(|e| e.to_string())?;
    if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
        t.completed = true;
        db.update_task(t).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_reminders(
    state: AppStateGuard<'_>,
) -> CmdResult<Vec<Reminder>> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.list_reminders().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_reminder(
    req:   NewReminder,
    state: AppStateGuard<'_>,
) -> CmdResult<Reminder> {
    let reminder = Reminder::new(req);
    let s        = state.lock().await;
    let db       = s.db.lock().await;
    db.insert_reminder(&reminder).map_err(|e| e.to_string())?;
    Ok(reminder)
}

#[tauri::command]
pub async fn delete_reminder(
    id:    String,
    state: AppStateGuard<'_>,
) -> CmdResult<()> {
    let s  = state.lock().await;
    let db = s.db.lock().await;
    db.delete_reminder(&id).map_err(|e| e.to_string())
}
