use std::collections::HashMap;

use chrono::{DateTime, Utc};
use reqwest;
use serde_json::Value;

pub struct RequestsResponseCache {
    pub response: Value,
    pub datetime: DateTime<Utc>,
}

struct MavenMetadata {
    release: String,
    latest: String,
    versions: Vec<String>,
}
