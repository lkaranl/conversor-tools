use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, serde::Serialize)]
pub struct JobStatus {
    pub id: String,
    pub status: String,    // "processing", "completed", "error"
    pub error: Option<String>,
    pub filename: String,
    pub compressed_filename: Option<String>,
}

pub struct AppStateInner {
    pub jobs: HashMap<String, JobStatus>,
}

pub type AppState = Arc<RwLock<AppStateInner>>;

pub fn new_state() -> AppState {
    Arc::new(RwLock::new(AppStateInner {
        jobs: HashMap::new(),
    }))
}
