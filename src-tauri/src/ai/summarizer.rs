use crate::ai::ollama::OllamaClient;
use crate::integrations::plugin::UnifiedEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub generated_at:    chrono::DateTime<chrono::Utc>,
    pub email_summary:   String,
    pub slack_summary:   String,
    pub jira_summary:    String,
    pub zoom_summary:    String,
    pub gchat_summary:   String,
    pub top_priorities:  Vec<PriorityItem>,
    pub motivational:    String,
    pub raw_event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityItem {
    pub title:      String,
    pub source:     String,
    pub url:        Option<String>,
    pub priority:   String,
    pub action:     String,
}

pub struct AISummarizer {
    ollama: OllamaClient,
}

const SYSTEM_PROMPT: &str =
    "You are Meridian, a concise executive assistant AI. \
     You summarize work activity clearly and professionally. \
     Be direct, use bullet points where helpful, avoid filler words. \
     Never invent information not in the provided data.";

impl AISummarizer {
    pub fn new() -> Self {
        Self { ollama: OllamaClient::new() }
    }

    pub async fn is_available(&self) -> bool {
        self.ollama.is_available().await
    }

    pub async fn model_ready(&self) -> bool {
        self.ollama.model_available().await
    }

    pub async fn generate_daily_summary(
        &self,
        events: &[UnifiedEvent],
    ) -> Result<DailySummary> {
        if events.is_empty() {
            return Ok(DailySummary::empty());
        }

        let email_events: Vec<_> = events.iter()
            .filter(|e| e.source == "gmail" || e.source == "outlook")
            .collect();
        let slack_events: Vec<_> = events.iter().filter(|e| e.source == "slack").collect();
        let jira_events:  Vec<_> = events.iter().filter(|e| e.source == "jira").collect();
        let zoom_events:  Vec<_> = events.iter().filter(|e| e.source == "zoom").collect();
        let gchat_events: Vec<_> = events.iter().filter(|e| e.source == "gchat").collect();

        let ollama_ok = self.is_available().await;

        let email_summary = if ollama_ok {
            self.summarize_group("emails", &email_events).await.unwrap_or_else(|_| basic_summary("email", &email_events))
        } else {
            basic_summary("email", &email_events)
        };

        let slack_summary = if ollama_ok {
            self.summarize_group("Slack messages", &slack_events).await.unwrap_or_else(|_| basic_summary("Slack", &slack_events))
        } else {
            basic_summary("Slack", &slack_events)
        };

        let jira_summary = if ollama_ok {
            self.summarize_group("Jira ticket updates", &jira_events).await.unwrap_or_else(|_| basic_summary("Jira", &jira_events))
        } else {
            basic_summary("Jira", &jira_events)
        };

        let zoom_summary = basic_summary("Zoom", &zoom_events);
        let gchat_summary = basic_summary("Google Chat", &gchat_events);

        let top_priorities = if ollama_ok {
            self.extract_priorities(events).await.unwrap_or_default()
        } else {
            default_priorities(events)
        };

        let motivational = if ollama_ok {
            self.get_motivational().await.unwrap_or_else(|_| default_quote())
        } else {
            default_quote()
        };

        Ok(DailySummary {
            generated_at:    chrono::Utc::now(),
            email_summary,
            slack_summary,
            jira_summary,
            zoom_summary,
            gchat_summary,
            top_priorities,
            motivational,
            raw_event_count: events.len(),
        })
    }

    async fn summarize_group(
        &self,
        category: &str,
        events: &[&UnifiedEvent],
    ) -> Result<String> {
        if events.is_empty() {
            return Ok(format!("No new {category}."));
        }

        let items: Vec<String> = events.iter().take(15).map(|e| {
            format!(
                "- From: {}\n  Subject: {}\n  Preview: {}",
                e.sender.as_deref().unwrap_or("Unknown"),
                e.title,
                &e.body[..e.body.len().min(200)]
            )
        }).collect();

        let user_msg = format!(
            "Summarize these {category} in 2-3 concise bullet points. \
             Focus only on action items, decisions needed, or important updates:\n\n{}",
            items.join("\n")
        );

        self.ollama.chat(SYSTEM_PROMPT, &user_msg, 0.3).await
    }

    async fn extract_priorities(&self, events: &[UnifiedEvent]) -> Result<Vec<PriorityItem>> {
        if events.is_empty() { return Ok(Vec::new()); }

        let items: Vec<String> = events.iter().take(25).map(|e| {
            format!("[{}] {} — {}", e.source, e.title, &e.body[..e.body.len().min(100)])
        }).collect();

        let user_msg = format!(
            "From these work items, identify the TOP 3 that need immediate attention.\n\
             Reply ONLY with valid JSON array, no markdown:\n\
             [{{\
               \"title\":\"...\",\
               \"source\":\"...\",\
               \"priority\":\"high|medium|low\",\
               \"action\":\"what to do\"\
             }}]\n\nItems:\n{}",
            items.join("\n")
        );

        let raw = self.ollama.chat(SYSTEM_PROMPT, &user_msg, 0.2).await?;
        let json_str = extract_json_array(&raw);

        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .unwrap_or_default();

        Ok(parsed.into_iter().map(|v| {
            let source_id = v["source"].as_str().unwrap_or("").to_string();
            let url = events.iter()
                .find(|e| e.source == source_id)
                .and_then(|e| e.url.clone());

            PriorityItem {
                title:    v["title"].as_str().unwrap_or("").to_string(),
                source:   source_id,
                url,
                priority: v["priority"].as_str().unwrap_or("medium").to_string(),
                action:   v["action"].as_str().unwrap_or("Review this item").to_string(),
            }
        }).collect())
    }

    async fn get_motivational(&self) -> Result<String> {
        let user_msg = "Give me one short motivational quote (one sentence, under 15 words) \
                        for a professional starting their workday. No attribution needed.";
        self.ollama.chat(SYSTEM_PROMPT, user_msg, 0.8).await
    }

    // ── Email assistant ───────────────────────────────────────────────────────

    pub async fn draft_email(&self, context: &str, instructions: &str) -> Result<String> {
        let user_msg = format!(
            "Draft a professional email.\nContext: {context}\nInstructions: {instructions}\n\
             Write the email body only, starting with the greeting."
        );
        self.ollama.chat(SYSTEM_PROMPT, &user_msg, 0.5).await
    }

    pub async fn rephrase_text(&self, text: &str, tone: &str) -> Result<String> {
        let user_msg = format!(
            "Rephrase the following text with a {tone} tone. \
             Keep the same meaning. Return only the rephrased text:\n\n{text}"
        );
        self.ollama.chat(SYSTEM_PROMPT, &user_msg, 0.4).await
    }

    pub async fn summarize_text(&self, text: &str) -> Result<String> {
        let user_msg = format!(
            "Summarize this in 2-3 bullet points. Be concise:\n\n{text}"
        );
        self.ollama.chat(SYSTEM_PROMPT, &user_msg, 0.3).await
    }

    pub async fn ask(&self, question: &str, context: &str) -> Result<String> {
        let user_msg = if context.is_empty() {
            question.to_string()
        } else {
            format!("Context:\n{context}\n\nQuestion: {question}")
        };
        self.ollama.chat(SYSTEM_PROMPT, &user_msg, 0.6).await
    }
}

// ── Fallbacks (no AI) ─────────────────────────────────────────────────────────

fn basic_summary(source: &str, events: &[&UnifiedEvent]) -> String {
    if events.is_empty() {
        return format!("No new {source} updates.");
    }
    let count = events.len();
    let items: Vec<String> = events.iter().take(3).map(|e| format!("• {}", e.title)).collect();
    format!("{count} new {source} item(s):\n{}", items.join("\n"))
}

fn default_priorities(events: &[UnifiedEvent]) -> Vec<PriorityItem> {
    events.iter()
        .filter(|e| e.priority.as_deref() == Some("high"))
        .take(3)
        .map(|e| PriorityItem {
            title:    e.title.clone(),
            source:   e.source.clone(),
            url:      e.url.clone(),
            priority: "high".to_string(),
            action:   "Review and respond".to_string(),
        })
        .collect()
}

fn default_quote() -> String {
    "Start each day with a clear mind and a focused plan.".to_string()
}

fn extract_json_array(text: &str) -> String {
    let start = text.find('[').unwrap_or(0);
    let end   = text.rfind(']').map(|i| i + 1).unwrap_or(text.len());
    if start < end { text[start..end].to_string() } else { "[]".to_string() }
}

impl DailySummary {
    pub fn empty() -> Self {
        Self {
            generated_at:    chrono::Utc::now(),
            email_summary:   "No new emails.".to_string(),
            slack_summary:   "No new Slack messages.".to_string(),
            jira_summary:    "No new Jira updates.".to_string(),
            zoom_summary:    "No upcoming meetings.".to_string(),
            gchat_summary:   "No new Google Chat messages.".to_string(),
            top_priorities:  Vec::new(),
            motivational:    default_quote(),
            raw_event_count: 0,
        }
    }
}
