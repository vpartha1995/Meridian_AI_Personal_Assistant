use crate::ai::summarizer::AISummarizer;
use crate::integrations::manager::IntegrationManager;
use crate::notifications::dispatcher::NotificationDispatcher;
use crate::storage::database::Database;
use crate::storage::keychain::Keychain;
use crate::tasks::reminder::ReminderEngine;
use anyhow::Result;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

pub struct AppState {
    pub db:           Arc<Mutex<Database>>,
    pub keychain:     Arc<Keychain>,
    pub integrations: Arc<Mutex<IntegrationManager>>,
    pub ai:           Arc<AISummarizer>,
    pub notifications:Arc<NotificationDispatcher>,
}

impl AppState {
    pub fn new(app: AppHandle) -> Result<Self> {
        // Resolve data directory
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("Cannot resolve app data dir: {e}"))?;
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("meridian.db");

        // Initialise keychain (derives DB key)
        let keychain = Arc::new(Keychain::new()?);
        let db_key   = keychain.get_or_create_db_key()?;

        // Open encrypted SQLite database
        let db = Arc::new(Mutex::new(Database::open(&db_path, &db_key)?));

        let notifications = Arc::new(NotificationDispatcher::new(app.clone()));
        let ai = Arc::new(AISummarizer::new());
        let integrations = Arc::new(Mutex::new(IntegrationManager::new(
            Arc::clone(&db),
            Arc::clone(&keychain),
        )));

        Ok(Self { db, keychain, integrations, ai, notifications })
    }

    pub async fn start_background_services(&mut self, app: AppHandle) -> Result<()> {
        // Start reminder engine
        let db_clone    = Arc::clone(&self.db);
        let notif_clone = Arc::clone(&self.notifications);
        tokio::spawn(async move {
            let engine = ReminderEngine::new(db_clone, notif_clone);
            engine.run().await;
        });

        // Start periodic integration sync (every 60 min)
        let integrations = Arc::clone(&self.integrations);
        let db_for_sync  = Arc::clone(&self.db);
        let ai_for_sync  = Arc::clone(&self.ai);
        tokio::spawn(async move {
            use tokio::time::{interval, Duration};
            let mut ticker = interval(Duration::from_secs(3600));
            loop {
                ticker.tick().await;
                log::info!("Running scheduled integration sync…");
                let mut mgr = integrations.lock().await;
                if let Err(e) = mgr.sync_all().await {
                    log::error!("Scheduled sync error: {e}");
                }
                // Re-generate summary after sync
                let db = db_for_sync.lock().await;
                let events = db.get_unprocessed_events().unwrap_or_default();
                drop(db);
                if !events.is_empty() {
                    match ai_for_sync.generate_daily_summary(&events).await {
                        Ok(summary) => {
                            let db = db_for_sync.lock().await;
                            if let Err(e) = db.save_summary(&summary) {
                                log::error!("Failed to save summary: {e}");
                            }
                        }
                        Err(e) => log::error!("Summary generation error: {e}"),
                    }
                }
            }
        });

        log::info!("Background services started");
        Ok(())
    }
}
