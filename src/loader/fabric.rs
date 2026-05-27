use serde::Deserialize;

use crate::{core::version::VersionJson, net::http, LauncherError, Result};

const FABRIC_META_BASE: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

pub fn latest_stable_loader(versions: &[FabricLoaderVersion]) -> Result<&FabricLoaderVersion> {
    versions
        .iter()
        .find(|version| version.stable)
        .ok_or_else(|| LauncherError::LoaderVersionNotFound {
            loader: crate::loader::LoaderKind::Fabric,
            version: "latest stable".to_string(),
        })
}

pub fn list_loader_versions() -> Result<Vec<FabricLoaderVersion>> {
    http::get_json(&format!("{FABRIC_META_BASE}/versions/loader"))
}

pub fn fetch_profile(minecraft_version: &str, loader_version: &str) -> Result<VersionJson> {
    http::get_json(&format!(
        "{FABRIC_META_BASE}/versions/loader/{minecraft_version}/{loader_version}/profile/json"
    ))
}
