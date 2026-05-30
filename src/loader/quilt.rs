//! Quilt loader metadata helpers.

use std::collections::HashMap;

use serde::Deserialize;

use crate::{core::version::VersionJson, net::http, LauncherError, Result};

const QUILT_META_BASE: &str = "https://meta.quiltmc.org/v3";

/// Quilt loader version entry from Quilt Meta.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct QuiltLoaderVersion {
    /// Maven coordinate.
    pub maven: String,
    /// Loader version string.
    pub version: String,
    /// Build number.
    pub build: i32,
    /// Maven version separator.
    pub separator: String,
    /// Optional loader file size.
    #[serde(default)]
    pub file_size: Option<i64>,
    /// Optional hashes keyed by algorithm.
    #[serde(default)]
    pub hashes: HashMap<String, String>,
}

/// Returns the first Quilt loader version from metadata.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the metadata list is empty.
pub fn latest_loader(versions: &[QuiltLoaderVersion]) -> Result<&QuiltLoaderVersion> {
    versions
        .first()
        .ok_or_else(|| LauncherError::LoaderVersionNotFound {
            loader: crate::loader::LoaderKind::Quilt,
            version: "latest".to_string(),
        })
}

/// Fetches Quilt loader versions.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the metadata request or decoding fails.
pub fn list_loader_versions() -> Result<Vec<QuiltLoaderVersion>> {
    http::get_json(&format!("{QUILT_META_BASE}/versions/loader"))
}

/// Fetches a Quilt loader profile for a Minecraft and loader version.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the metadata request or decoding fails.
pub fn fetch_profile(minecraft_version: &str, loader_version: &str) -> Result<VersionJson> {
    http::get_json(&format!(
        "{QUILT_META_BASE}/versions/loader/{minecraft_version}/{loader_version}/profile/json"
    ))
}
