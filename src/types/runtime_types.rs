use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeListJsonEntryManifest {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeListJsonEntry {
    /// keys: {group, progress}
    pub availability: HashMap<String, i32>,
    pub manifest: RuntimeListJsonEntryManifest,
    /// keys: {name, released}
    pub version: HashMap<String, String>,
}

pub type RuntimeListJson = HashMap<String, HashMap<String, Vec<RuntimeListJsonEntry>>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformManifestJsonFileDownloads {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformManifestJsonFile {
    /// keys: {lzma, raw}
    pub downloads: HashMap<String, PlatformManifestJsonFileDownloads>,
    /// keys: {file, direactory, link}
    pub r#type: String,
    pub executable: bool,
    pub target: String,
}

type PlatformManifestJson = HashMap<String, PlatformManifestJsonFile>;
