//! Fabric loader metadata helpers.

use serde::Deserialize;

use crate::{core::version::VersionJson, net::http, LauncherError, Result};

const FABRIC_META_BASE: &str = "https://meta.fabricmc.net/v2";

/// Fabric loader version entry from Fabric Meta.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FabricLoaderVersion {
    /// Maven version separator.
    pub separator: String,
    /// Build number.
    pub build: i32,
    /// Maven coordinate.
    pub maven: String,
    /// Loader version string.
    pub version: String,
    /// Whether Fabric marks this loader as stable.
    pub stable: bool,
}

/// Returns the first stable Fabric loader version from metadata.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if no stable loader is available.
pub fn latest_stable_loader(versions: &[FabricLoaderVersion]) -> Result<&FabricLoaderVersion> {
    versions
        .iter()
        .find(|version| version.stable)
        .ok_or_else(|| LauncherError::LoaderVersionNotFound {
            loader: crate::loader::LoaderKind::Fabric,
            version: "latest stable".to_string(),
        })
}

/// Fetches Fabric loader versions.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the metadata request or decoding fails.
pub fn list_loader_versions() -> Result<Vec<FabricLoaderVersion>> {
    http::get_json(&format!("{FABRIC_META_BASE}/versions/loader"))
}

/// Fetches a Fabric loader profile for a Minecraft and loader version.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the metadata request or decoding fails.
pub fn fetch_profile(minecraft_version: &str, loader_version: &str) -> Result<VersionJson> {
    http::get_json(&format!(
        "{FABRIC_META_BASE}/versions/loader/{minecraft_version}/{loader_version}/profile/json"
    ))
}
