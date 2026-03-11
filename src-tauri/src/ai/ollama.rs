use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

pub const OLLAMA_BASE: &str = "http://127.0.0.1:11434";
pub const DEFAULT_MODEL: &str = "phi3:mini";

/// Holds the spawned Ollama sidecar child so it can be killed on exit.
static OLLAMA_CHILD: Mutex<Option<CommandChild>> = Mutex::new(None);

// ── Sidecar lifecycle ─────────────────────────────────────────────────────────

/// Ensures the Ollama daemon is running.
/// Tries the bundled sidecar first; falls back to a system `ollama serve`.
/// Spawns a background thread — does not block the caller.
pub fn ensure_ollama_running(app: AppHandle) {
    std::thread::spawn(move || {
        // Already running? Nothing to do.
        if port_open() {
            log::info!("Ollama already running on 127.0.0.1:11434");
            return;
        }

        // Attempt to start the bundled sidecar binary.
        let started = match app.shell().sidecar("ollama") {
            Ok(cmd) => match cmd.args(["serve"]).spawn() {
                Ok((_rx, child)) => {
                    *OLLAMA_CHILD.lock().unwrap() = Some(child);
                    log::info!("Started bundled Ollama sidecar");
                    true
                }
                Err(e) => {
                    log::warn!("Bundled Ollama sidecar spawn failed: {e}");
                    false
                }
            },
            Err(e) => {
                log::debug!("No bundled Ollama sidecar configured: {e}");
                false
            }
        };

        // Fall back to system-installed ollama.
        if !started {
            try_system_ollama();
        }

        // Wait up to 10 s for the server to be ready.
        for i in 0..20 {
            std::thread::sleep(Duration::from_millis(500));
            if port_open() {
                log::info!("Ollama ready after ~{} ms", (i + 1) * 500);
                return;
            }
        }
        log::warn!("Ollama did not become ready within 10 s");
    });
}

/// Kills the bundled sidecar (if we started it). Called on app exit.
pub fn stop_ollama() {
    if let Some(child) = OLLAMA_CHILD.lock().unwrap().take() {
        let _ = child.kill();
        log::info!("Stopped bundled Ollama sidecar");
    }
}

fn port_open() -> bool {
    std::net::TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        Duration::from_millis(300),
    )
    .is_ok()
}

fn try_system_ollama() {
    match std::process::Command::new("ollama")
        .arg("serve")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => log::info!("Started system Ollama"),
        Err(e) => log::debug!("System Ollama not available: {e}"),
    }
}

// ── Model pull with streaming progress ───────────────────────────────────────

#[derive(Clone, Serialize)]
struct PullProgressPayload {
    model:     String,
    status:    String,
    total:     u64,
    completed: u64,
    percent:   u8,
}

/// Stream-pulls an Ollama model and emits `ollama://pull-progress` events
/// to the frontend as each NDJSON line arrives.
pub async fn pull_model_with_progress(app: &AppHandle, model: &str) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(7200)) // large models can take a while
        .build()?;

    #[derive(Serialize)]
    struct PullReq<'a> {
        model:  &'a str,
        stream: bool,
    }

    let response = client
        .post(format!("{OLLAMA_BASE}/api/pull"))
        .json(&PullReq { model, stream: true })
        .send()
        .await
        .context("Failed to connect to Ollama for model pull")?;

    #[derive(Deserialize)]
    struct PullLine {
        status:    Option<String>,
        total:     Option<u64>,
        completed: Option<u64>,
    }

    let mut stream = response.bytes_stream();
    let mut buf    = String::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.context("Error reading pull stream")?;
        buf.push_str(&String::from_utf8_lossy(&bytes));

        // Drain complete newline-terminated JSON objects.
        while let Some(nl) = buf.find('\n') {
            let line = buf[..nl].trim().to_string();
            buf = buf[nl + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Ok(pl) = serde_json::from_str::<PullLine>(&line) {
                let total     = pl.total.unwrap_or(0);
                let completed = pl.completed.unwrap_or(0);
                let percent   = if total > 0 {
                    ((completed as f64 / total as f64) * 100.0).min(100.0) as u8
                } else {
                    0
                };
                let status = pl.status.clone().unwrap_or_default();
                let done   = status == "success";

                let _ = app.emit(
                    "ollama://pull-progress",
                    PullProgressPayload {
                        model:  model.to_string(),
                        status: status.clone(),
                        total,
                        completed,
                        percent,
                    },
                );

                log::debug!("Pull {model}: {status} {completed}/{total} ({percent}%)");

                if done {
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}

// ── OllamaClient (HTTP wrapper) ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name:   String,
    pub size:   u64,
    pub digest: String,
}

pub struct OllamaClient {
    client:       Client,
    pub base_url: String,
    pub model:    String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            base_url: OLLAMA_BASE.to_string(),
            model:    DEFAULT_MODEL.to_string(),
        }
    }

    /// Returns `true` when the Ollama daemon is reachable.
    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Returns installed models.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        #[derive(Deserialize)]
        struct TagsResp {
            models: Vec<ModelInfo>,
        }
        let resp: TagsResp = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .context("Could not reach Ollama")?
            .json()
            .await?;
        Ok(resp.models)
    }

    /// Returns `true` when the target model is already pulled.
    pub async fn model_available(&self) -> bool {
        self.list_models()
            .await
            .map(|models| models.iter().any(|m| m.name.starts_with(&self.model)))
            .unwrap_or(false)
    }

    /// Generate a completion (non-streaming).
    pub async fn generate(
        &self,
        prompt:      &str,
        temperature: f32,
        max_tokens:  i32,
    ) -> Result<String> {
        #[derive(Serialize)]
        struct Req<'a> {
            model:   &'a str,
            prompt:  &'a str,
            stream:  bool,
            options: Opts,
        }
        #[derive(Serialize)]
        struct Opts {
            temperature: f32,
            num_predict: i32,
        }
        #[derive(Deserialize)]
        struct Resp {
            response: String,
        }

        let body = Req {
            model:   &self.model,
            prompt,
            stream:  false,
            options: Opts { temperature, num_predict: max_tokens },
        };

        let resp: Resp = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&body)
            .send()
            .await
            .context("Ollama generate request failed")?
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(resp.response.trim().to_string())
    }

    /// Chat-style completion using the `/api/chat` endpoint.
    pub async fn chat(
        &self,
        system:      &str,
        user:        &str,
        temperature: f32,
    ) -> Result<String> {
        #[derive(Serialize)]
        struct Req<'a> {
            model:    &'a str,
            messages: Vec<Msg<'a>>,
            stream:   bool,
            options:  Opts,
        }
        #[derive(Serialize)]
        struct Msg<'a> {
            role:    &'a str,
            content: &'a str,
        }
        #[derive(Serialize)]
        struct Opts {
            temperature: f32,
            num_predict: i32,
        }
        #[derive(Deserialize)]
        struct Resp {
            message: RespMsg,
        }
        #[derive(Deserialize)]
        struct RespMsg {
            content: String,
        }

        let body = Req {
            model:    &self.model,
            messages: vec![
                Msg { role: "system", content: system },
                Msg { role: "user",   content: user   },
            ],
            stream:  false,
            options: Opts { temperature, num_predict: 1024 },
        };

        let resp: Resp = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .context("Ollama chat request failed")?
            .json()
            .await?;

        Ok(resp.message.content.trim().to_string())
    }
}
