//! Vanilla version metadata loading and complete file installation.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    compatibility::{apply_compatibility, CompatibilityPolicy},
    core::version::VersionJson,
    net::{download::execute_plan, http},
    platform::Platform,
    progress::ProgressReporter,
    LauncherError, Result,
};

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Deserialize)]
struct VersionManifest {
    versions: Vec<VersionManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct VersionManifestEntry {
    id: String,
    url: String,
}

/// Fetches a vanilla Minecraft version JSON from Mojang's version manifest.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the manifest cannot be fetched, the
/// version id is unknown, or the version JSON cannot be decoded.
pub fn fetch_vanilla_version(version_id: &str) -> Result<VersionJson> {
    let manifest: VersionManifest = http::get_json(VERSION_MANIFEST_URL)?;
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == version_id)
        .ok_or_else(|| LauncherError::InvalidVersionId {
            id: version_id.to_string(),
        })?;

    http::get_json(&entry.url)
}

/// Returns the canonical local path for a version JSON file.
pub fn version_json_path(minecraft_dir: impl AsRef<Path>, version_id: &str) -> PathBuf {
    minecraft_dir
        .as_ref()
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"))
}

/// Writes a version JSON to the standard local profile path.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the version has no `id`, the parent
/// directory cannot be created, or JSON serialization fails.
pub fn write_version_json(
    minecraft_dir: impl AsRef<Path>,
    version: &VersionJson,
) -> Result<PathBuf> {
    let version_id = version
        .id
        .as_deref()
        .ok_or_else(|| LauncherError::MissingField {
            context: "version json".to_string(),
            field: "id".to_string(),
        })?;
    let path = version_json_path(minecraft_dir, version_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_vec_pretty(version)?)?;
    Ok(path)
}

/// Reads a version JSON without merging inherited parent metadata.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the file cannot be read or decoded.
pub fn read_version_json(minecraft_dir: impl AsRef<Path>, version_id: &str) -> Result<VersionJson> {
    let path = version_json_path(minecraft_dir, version_id);
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

/// Reads a version JSON and recursively merges any `inheritsFrom` parents.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if this profile or any parent profile cannot
/// be read or decoded.
pub fn load_version_json(minecraft_dir: impl AsRef<Path>, version_id: &str) -> Result<VersionJson> {
    load_version_json_inner(minecraft_dir.as_ref(), version_id)
}

/// Installs client jar, libraries, assets, and native libraries for a version.
///
/// Compatibility patches are applied automatically for the current platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if planning, downloads, checksums, asset
/// decoding, or native extraction fails.
pub fn install_version_files(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
    reporter: &mut dyn ProgressReporter,
) -> Result<()> {
    install_version_files_for_platform(
        version,
        minecraft_dir,
        Platform::current(),
        CompatibilityPolicy::Auto,
        reporter,
    )
}

/// Installs version files for an explicit platform and compatibility policy.
///
/// This is useful for tests and custom launchers that need deterministic
/// cross-platform planning.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if planning, downloads, checksums, asset
/// decoding, or native extraction fails.
pub fn install_version_files_for_platform(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
    platform: Platform,
    compatibility: CompatibilityPolicy,
    reporter: &mut dyn ProgressReporter,
) -> Result<()> {
    let minecraft_dir = minecraft_dir.as_ref();
    let compatibility = apply_compatibility(version, platform, compatibility);
    let version = &compatibility.version;
    let version_id = version
        .id
        .as_deref()
        .ok_or_else(|| LauncherError::MissingField {
            context: "version json".to_string(),
            field: "id".to_string(),
        })?;

    let plan = crate::install::vanilla::plan_vanilla_downloads_for_platform(
        version,
        minecraft_dir,
        platform,
        CompatibilityPolicy::Disabled,
    )?;
    execute_plan(&plan, reporter)?;
    crate::install::assets::install_assets(version, minecraft_dir, reporter)?;
    crate::install::natives::extract_natives_for_platform(
        &version.libraries,
        minecraft_dir,
        version_id,
        platform,
    )?;
    Ok(())
}

fn load_version_json_inner(minecraft_dir: &Path, version_id: &str) -> Result<VersionJson> {
    let version = read_version_json(minecraft_dir, version_id)?;
    let Some(parent_id) = version.inherits_from.clone() else {
        return Ok(version);
    };
    let parent = load_version_json_inner(minecraft_dir, &parent_id)?;
    Ok(parent.merge_child(&version))
}
