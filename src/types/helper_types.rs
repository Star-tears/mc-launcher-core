use chrono::{DateTime, Utc};

pub struct RequestsResponseCache {
    pub response: String,
    pub datetime: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MavenMetadata {
    pub release: String,
    pub latest: String,
    pub versions: Vec<String>,
}
