use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id:              String,
    pub title:           String,
    pub description:     Option<String>,
    pub due_at:          Option<String>,
    pub completed:       bool,
    pub priority:        String,
    pub recurrence_rule: Option<String>,
    pub source_url:      Option<String>,
    pub created_at:      String,
    pub updated_at:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTask {
    pub title:       String,
    pub description: Option<String>,
    pub due_at:      Option<String>,
    pub priority:    Option<String>,
    pub source_url:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id:      String,
    pub task_id: Option<String>,
    pub fire_at: String,
    pub message: String,
    pub fired:   bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReminder {
    pub task_id: Option<String>,
    pub fire_at: String,
    pub message: String,
}

impl Task {
    pub fn new(req: NewTask) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id:              uuid::Uuid::new_v4().to_string(),
            title:           req.title,
            description:     req.description,
            due_at:          req.due_at,
            completed:       false,
            priority:        req.priority.unwrap_or_else(|| "medium".to_string()),
            recurrence_rule: None,
            source_url:      req.source_url,
            created_at:      now.clone(),
            updated_at:      now,
        }
    }
}

impl Reminder {
    pub fn new(req: NewReminder) -> Self {
        Self {
            id:      uuid::Uuid::new_v4().to_string(),
            task_id: req.task_id,
            fire_at: req.fire_at,
            message: req.message,
            fired:   false,
        }
    }
}
