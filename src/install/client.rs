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

pub fn version_json_path(minecraft_dir: impl AsRef<Path>, version_id: &str) -> PathBuf {
    minecraft_dir
        .as_ref()
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"))
}

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

pub fn read_version_json(minecraft_dir: impl AsRef<Path>, version_id: &str) -> Result<VersionJson> {
    let path = version_json_path(minecraft_dir, version_id);
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub fn load_version_json(minecraft_dir: impl AsRef<Path>, version_id: &str) -> Result<VersionJson> {
    load_version_json_inner(minecraft_dir.as_ref(), version_id)
}

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
