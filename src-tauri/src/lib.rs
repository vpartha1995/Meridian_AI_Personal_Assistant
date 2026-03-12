pub mod ai;
pub mod commands;
pub mod integrations;
pub mod notifications;
pub mod state;
pub mod storage;
pub mod tasks;

use ai::ollama::{ensure_ollama_running, stop_ollama};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[cfg(mobile)]
mod mobile;

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Start Ollama daemon (bundled sidecar or system install) in background.
            ensure_ollama_running(app_handle.clone());

            // Initialize app state (DB, keychain, services)
            let state = AppState::new(app_handle.clone())
                .expect("Failed to initialize AppState");
            let state = Arc::new(tokio::sync::Mutex::new(state));
            app.manage(state.clone());

            // Register global shortcut CTRL+SPACE for overlay
            let handle = app_handle.clone();
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
            app.global_shortcut()
                .on_shortcut(shortcut, move |_app, _shortcut, _event| {
                    if let Some(overlay) = handle.get_webview_window("overlay") {
                        if overlay.is_visible().unwrap_or(false) {
                            let _ = overlay.hide();
                        } else {
                            let _ = overlay.show();
                            let _ = overlay.set_focus();
                        }
                    }
                })
                .ok();

            // Spawn background services
            let state_clone = state.clone();
            let handle2 = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut locked = state_clone.lock().await;
                if let Err(e) = locked.start_background_services(handle2).await {
                    log::error!("Background services error: {e}");
                }
            });

            // System tray
            #[cfg(desktop)]
            {
                use tauri::menu::{Menu, MenuItem};
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};

                let show_i = MenuItem::with_id(app, "show", "Show Meridian", true, None::<&str>)?;
                let quit_i  = MenuItem::with_id(app, "quit", "Quit",          true, None::<&str>)?;
                let menu    = Menu::with_items(app, &[&show_i, &quit_i])?;

                let handle3 = app_handle.clone();
                TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .on_menu_event(move |_app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(win) = handle3.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "quit" => std::process::exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { .. } = event {
                            let app = tray.app_handle();
                            if let Some(win) = app.get_webview_window("main") {
                                if win.is_visible().unwrap_or(false) {
                                    let _ = win.hide();
                                } else {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                }
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Integration commands
            commands::integration::list_integrations,
            commands::integration::connect_integration,
            commands::integration::disconnect_integration,
            commands::integration::get_integration_status,
            commands::integration::trigger_sync,
            // Summary commands
            commands::summary::get_todays_summary,
            commands::summary::generate_daily_summary,
            commands::summary::get_summary_history,
            // Task commands
            commands::tasks::list_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::delete_task,
            commands::tasks::complete_task,
            commands::tasks::list_reminders,
            commands::tasks::create_reminder,
            commands::tasks::delete_reminder,
            // AI commands
            commands::ai::check_ollama_status,
            commands::ai::pull_model,
            commands::ai::draft_email,
            commands::ai::rephrase_text,
            commands::ai::ask_assistant,
            commands::ai::summarize_text,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_theme,
            commands::settings::set_theme,
            commands::settings::log_health_action,
            commands::settings::set_oauth_creds,
            commands::settings::get_oauth_client_id,
            commands::settings::delete_oauth_creds,
            // Window
            commands::window::show_overlay,
            commands::window::hide_overlay,
            commands::window::open_url,
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                stop_ollama();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Meridian");
}
