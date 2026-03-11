pub mod ai;
pub mod integration;
pub mod settings;
pub mod summary;
pub mod tasks;
pub mod window;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self { message: e.to_string() }
    }
}

pub type CmdResult<T> = Result<T, String>;

pub fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}
