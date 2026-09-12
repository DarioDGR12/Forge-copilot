use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub pending: tokio::sync::Mutex<HashMap<String, oneshot::Sender<bool>>>,
    pub runs: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(db: Connection) -> crate::error::AppResult<Self> {
        Ok(Self {
            db: Mutex::new(db),
            pending: tokio::sync::Mutex::new(HashMap::new()),
            runs: Mutex::new(HashMap::new()),
            http: reqwest::Client::builder()
                .user_agent("Forge-Copilot/0.1")
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
        })
    }
}
