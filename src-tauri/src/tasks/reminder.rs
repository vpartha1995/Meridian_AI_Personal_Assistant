use crate::notifications::dispatcher::NotificationDispatcher;
use crate::storage::database::Database;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub struct ReminderEngine {
    db:      Arc<Mutex<Database>>,
    notifs:  Arc<NotificationDispatcher>,
}

impl ReminderEngine {
    pub fn new(db: Arc<Mutex<Database>>, notifs: Arc<NotificationDispatcher>) -> Self {
        Self { db, notifs }
    }

    /// Long-running loop — spawn with `tokio::spawn`.
    pub async fn run(self) {
        let mut check_ticker  = interval(Duration::from_secs(30));
        let mut nudge_ticker  = interval(Duration::from_secs(1800)); // every 30 min
        nudge_ticker.tick().await; // skip first immediate tick

        loop {
            tokio::select! {
                _ = check_ticker.tick() => {
                    if let Err(e) = self.fire_due(&self.db, &self.notifs).await {
                        log::error!("Reminder fire error: {e}");
                    }
                }
                _ = nudge_ticker.tick() => {
                    self.health_nudge().await;
                }
            }
        }
    }

    async fn fire_due(
        &self,
        db:    &Arc<Mutex<Database>>,
        notifs:&Arc<NotificationDispatcher>,
    ) -> anyhow::Result<()> {
        // Respect the global notifications toggle before firing any reminder.
        let notifications_enabled = {
            let db = db.lock().await;
            Self::read_notifications_enabled(&db)
        };
        if !notifications_enabled {
            return Ok(());
        }

        let reminders = {
            let db = db.lock().await;
            db.get_due_reminders()?
        };

        for r in reminders {
            notifs.send("Meridian Reminder", &r.message).await;
            let db = db.lock().await;
            let _ = db.mark_reminder_fired(&r.id);
        }
        Ok(())
    }

    async fn health_nudge(&self) {
        use chrono::Timelike;

        // Read settings first; hold the lock briefly.
        let (health_nudges_on, notifications_on, work_start, work_end) = {
            let db = self.db.lock().await;
            Self::read_nudge_settings(&db)
        };

        if !health_nudges_on || !notifications_on {
            return;
        }

        let hour = chrono::Local::now().hour();
        // Use the user-configured working hours instead of hard-coded 9–18.
        if !(work_start..=work_end).contains(&hour) {
            return;
        }

        let db = self.db.lock().await;
        let last_nudge = db.get_setting("last_health_nudge")
            .unwrap_or_default()
            .unwrap_or_default();

        let should_nudge = if last_nudge.is_empty() {
            true
        } else {
            let last: chrono::DateTime<chrono::Utc> =
                last_nudge.parse().unwrap_or_else(|_| chrono::Utc::now());
            chrono::Utc::now().signed_duration_since(last).num_minutes() >= 90
        };

        if should_nudge {
            drop(db);
            let nudges = [
                ("Stay Hydrated", "Time to drink some water. Hydration boosts focus by up to 30%."),
                ("Take a Break",  "You've been at it for a while. Stand up and stretch for 2 minutes."),
                ("Check Posture", "Sit up straight, relax your shoulders. Your back will thank you."),
                ("Move Around",   "A short walk or stretch every 90 minutes improves productivity."),
            ];
            let idx = (chrono::Utc::now().timestamp() / 1800 % nudges.len() as i64) as usize;
            let (title, body) = nudges[idx];
            self.notifs.send(title, body).await;

            let db = self.db.lock().await;
            let _ = db.set_setting("last_health_nudge", &chrono::Utc::now().to_rfc3339());
        }
    }

    // ── Settings helpers ──────────────────────────────────────────────────────

    fn read_notifications_enabled(db: &Database) -> bool {
        Self::parse_settings(db)
            .map(|s| s.notifications)
            .unwrap_or(true)
    }

    fn read_nudge_settings(db: &Database) -> (bool, bool, u32, u32) {
        Self::parse_settings(db)
            .map(|s| (
                s.health_nudges,
                s.notifications,
                s.working_hours_start as u32,
                s.working_hours_end   as u32,
            ))
            .unwrap_or((true, true, 9, 18))
    }

    fn parse_settings(db: &Database) -> Option<ReminderSettings> {
        let json = db.get_setting("app_settings").ok()??;
        serde_json::from_str::<ReminderSettings>(&json).ok()
    }
}

/// Minimal subset of AppSettings needed by the reminder engine.
#[derive(serde::Deserialize)]
struct ReminderSettings {
    #[serde(default = "default_true")]
    notifications:       bool,
    #[serde(default = "default_true")]
    health_nudges:       bool,
    #[serde(default = "default_start")]
    working_hours_start: u8,
    #[serde(default = "default_end")]
    working_hours_end:   u8,
}

fn default_true()  -> bool { true }
fn default_start() -> u8   { 9 }
fn default_end()   -> u8   { 18 }
