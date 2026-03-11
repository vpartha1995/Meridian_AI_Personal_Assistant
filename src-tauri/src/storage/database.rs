use crate::ai::summarizer::DailySummary;
use crate::integrations::plugin::UnifiedEvent;
use crate::tasks::manager::{Reminder, Task};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &PathBuf, _key: &[u8; 32]) -> Result<Self> {
        let conn = Connection::open(path)
            .context("Cannot open SQLite database")?;

        // Performance tuning
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous   = NORMAL;
             PRAGMA foreign_keys  = ON;
             PRAGMA cache_size    = -8000;",
        )
        .context("PRAGMA setup failed")?;

        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS events (
                id          TEXT PRIMARY KEY,
                source      TEXT NOT NULL,
                title       TEXT NOT NULL,
                body        TEXT NOT NULL DEFAULT '',
                sender      TEXT,
                url         TEXT,
                received_at TEXT NOT NULL,
                priority    TEXT,
                processed   INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS daily_summaries (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                date        TEXT NOT NULL UNIQUE,
                content     TEXT NOT NULL,
                event_count INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id               TEXT PRIMARY KEY,
                title            TEXT NOT NULL,
                description      TEXT,
                due_at           TEXT,
                completed        INTEGER NOT NULL DEFAULT 0,
                priority         TEXT    NOT NULL DEFAULT 'medium',
                recurrence_rule  TEXT,
                source_url       TEXT,
                created_at       TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS reminders (
                id       TEXT PRIMARY KEY,
                task_id  TEXT,
                fire_at  TEXT NOT NULL,
                message  TEXT NOT NULL,
                fired    INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS settings (
                key        TEXT PRIMARY KEY,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS sync_cursors (
                integration_id TEXT PRIMARY KEY,
                cursor_value   TEXT NOT NULL,
                last_sync      TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_events_source     ON events(source);
            CREATE INDEX IF NOT EXISTS idx_events_received   ON events(received_at);
            CREATE INDEX IF NOT EXISTS idx_events_processed  ON events(processed);
            CREATE INDEX IF NOT EXISTS idx_tasks_due         ON tasks(due_at);
            CREATE INDEX IF NOT EXISTS idx_reminders_fire    ON reminders(fire_at, fired);
        ").context("Migration failed")
    }

    // ── Events ───────────────────────────────────────────────────────────────

    pub fn upsert_event(&self, e: &UnifiedEvent) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events(id,source,title,body,sender,url,received_at,priority,processed)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,0)
             ON CONFLICT(id) DO NOTHING",
            params![
                e.id, e.source, e.title, e.body,
                e.sender, e.url,
                e.received_at.to_rfc3339(),
                e.priority.as_deref()
            ],
        )?;
        Ok(())
    }

    pub fn get_unprocessed_events(&self) -> Result<Vec<UnifiedEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,source,title,body,sender,url,received_at,priority
             FROM events WHERE processed=0
             ORDER BY received_at DESC LIMIT 200",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UnifiedEvent {
                id:          row.get(0)?,
                source:      row.get(1)?,
                title:       row.get(2)?,
                body:        row.get(3)?,
                sender:      row.get(4)?,
                url:         row.get(5)?,
                received_at: row.get::<_, String>(6)?
                    .parse()
                    .unwrap_or_else(|_| chrono::Utc::now()),
                priority:    row.get(7)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
           .context("Failed to fetch unprocessed events")
    }

    pub fn mark_events_processed(&self, ids: &[String]) -> Result<()> {
        for id in ids {
            self.conn.execute(
                "UPDATE events SET processed=1 WHERE id=?1",
                params![id],
            )?;
        }
        Ok(())
    }

    // ── Daily summaries ───────────────────────────────────────────────────────

    pub fn save_summary(&self, summary: &DailySummary) -> Result<()> {
        let content = serde_json::to_string(summary)?;
        let date    = summary.generated_at.format("%Y-%m-%d").to_string();
        self.conn.execute(
            "INSERT INTO daily_summaries(date, content, event_count)
             VALUES(?1,?2,?3)
             ON CONFLICT(date) DO UPDATE SET content=excluded.content,
             event_count=excluded.event_count",
            params![date, content, summary.raw_event_count],
        )?;
        Ok(())
    }

    pub fn get_todays_summary(&self) -> Result<Option<DailySummary>> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = self.conn.prepare(
            "SELECT content FROM daily_summaries WHERE date=?1",
        )?;
        let result = stmt
            .query_row(params![today], |row| row.get::<_, String>(0))
            .optional()?;
        match result {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub fn get_summary_history(&self, limit: u32) -> Result<Vec<DailySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM daily_summaries ORDER BY date DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            if let Ok(json) = r {
                if let Ok(s) = serde_json::from_str(&json) {
                    out.push(s);
                }
            }
        }
        Ok(out)
    }

    // ── Tasks ─────────────────────────────────────────────────────────────────

    pub fn list_tasks(&self, include_completed: bool) -> Result<Vec<Task>> {
        let sql = if include_completed {
            "SELECT id,title,description,due_at,completed,priority,recurrence_rule,source_url,created_at,updated_at FROM tasks ORDER BY due_at ASC NULLS LAST"
        } else {
            "SELECT id,title,description,due_at,completed,priority,recurrence_rule,source_url,created_at,updated_at FROM tasks WHERE completed=0 ORDER BY due_at ASC NULLS LAST"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(Task {
                id:               row.get(0)?,
                title:            row.get(1)?,
                description:      row.get(2)?,
                due_at:           row.get(3)?,
                completed:        row.get::<_,i32>(4)? != 0,
                priority:         row.get(5)?,
                recurrence_rule:  row.get(6)?,
                source_url:       row.get(7)?,
                created_at:       row.get(8)?,
                updated_at:       row.get(9)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().context("list_tasks")
    }

    pub fn insert_task(&self, t: &Task) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tasks(id,title,description,due_at,completed,priority,recurrence_rule,source_url,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                t.id, t.title, t.description, t.due_at,
                t.completed as i32, t.priority,
                t.recurrence_rule, t.source_url,
                t.created_at, t.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn update_task(&self, t: &Task) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET title=?2,description=?3,due_at=?4,completed=?5,priority=?6,
             recurrence_rule=?7,source_url=?8,updated_at=?9 WHERE id=?1",
            params![
                t.id, t.title, t.description, t.due_at,
                t.completed as i32, t.priority,
                t.recurrence_rule, t.source_url,
                chrono::Local::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Reminders ─────────────────────────────────────────────────────────────

    pub fn get_due_reminders(&self) -> Result<Vec<Reminder>> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT id,task_id,fire_at,message FROM reminders
             WHERE fired=0 AND fire_at<=?1",
        )?;
        let rows = stmt.query_map(params![now], |row| {
            Ok(Reminder {
                id:      row.get(0)?,
                task_id: row.get(1)?,
                fire_at: row.get(2)?,
                message: row.get(3)?,
                fired:   false,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().context("get_due_reminders")
    }

    pub fn mark_reminder_fired(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE reminders SET fired=1 WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn insert_reminder(&self, r: &Reminder) -> Result<()> {
        self.conn.execute(
            "INSERT INTO reminders(id,task_id,fire_at,message) VALUES(?1,?2,?3,?4)",
            params![r.id, r.task_id, r.fire_at, r.message],
        )?;
        Ok(())
    }

    pub fn list_reminders(&self) -> Result<Vec<Reminder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,task_id,fire_at,message,fired FROM reminders
             WHERE fired=0 ORDER BY fire_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Reminder {
                id:      row.get(0)?,
                task_id: row.get(1)?,
                fire_at: row.get(2)?,
                message: row.get(3)?,
                fired:   row.get::<_,i32>(4)? != 0,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().context("list_reminders")
    }

    pub fn delete_reminder(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM reminders WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Settings ──────────────────────────────────────────────────────────────

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM settings WHERE key=?1",
        )?;
        stmt.query_row(params![key], |row| row.get(0))
            .optional()
            .context("get_setting")
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
            params![key, value],
        )?;
        Ok(())
    }

    // ── Sync cursors ──────────────────────────────────────────────────────────

    pub fn get_sync_cursor(&self, id: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT cursor_value FROM sync_cursors WHERE integration_id=?1",
        )?;
        stmt.query_row(params![id], |row| row.get(0))
            .optional()
            .context("get_sync_cursor")
    }

    pub fn upsert_sync_cursor(&self, id: &str, cursor: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sync_cursors(integration_id,cursor_value,last_sync)
             VALUES(?1,?2,?3)
             ON CONFLICT(integration_id) DO UPDATE SET
             cursor_value=excluded.cursor_value, last_sync=excluded.last_sync",
            params![id, cursor, now],
        )?;
        Ok(())
    }
}

// Helper: rusqlite optional row
trait OptRow<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}
impl<T> OptRow<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
