use std::collections::HashMap;

use serde::Deserialize;

use crate::{core::version::VersionJson, net::http, LauncherError, Result};

const QUILT_META_BASE: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct QuiltLoaderVersion {
    pub maven: String,
    pub version: String,
    pub build: i32,
    pub separator: String,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub hashes: HashMap<String, String>,
}

pub fn latest_loader(versions: &[QuiltLoaderVersion]) -> Result<&QuiltLoaderVersion> {
    versions
        .first()
        .ok_or_else(|| LauncherError::LoaderVersionNotFound {
            loader: crate::loader::LoaderKind::Quilt,
            version: "latest".to_string(),
        })
}

pub fn list_loader_versions() -> Result<Vec<QuiltLoaderVersion>> {
    http::get_json(&format!("{QUILT_META_BASE}/versions/loader"))
}

pub fn fetch_profile(minecraft_version: &str, loader_version: &str) -> Result<VersionJson> {
    http::get_json(&format!(
        "{QUILT_META_BASE}/versions/loader/{minecraft_version}/{loader_version}/profile/json"
    ))
}
