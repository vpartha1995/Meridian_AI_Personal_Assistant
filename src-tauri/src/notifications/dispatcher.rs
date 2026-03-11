use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub struct NotificationDispatcher {
    app: AppHandle,
}

impl NotificationDispatcher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn send(&self, title: &str, body: &str) {
        if let Err(e) = self.app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            log::warn!("Notification failed: {e}");
        }
    }
}
