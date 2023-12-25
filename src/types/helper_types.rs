use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct RequestsResponseCache {
    pub response: Value,
    pub datetime: DateTime<Utc>,
}

pub struct MavenMetadata {
    pub release: String,
    pub latest: String,
    pub versions: Vec<String>,
}
