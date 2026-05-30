//! Download planning for vanilla client files.

use std::path::Path;

use crate::{
    compatibility::{apply_compatibility, CompatibilityPolicy},
    core::version::VersionJson,
    net::download::{Checksum, DownloadPlan, DownloadTask},
    platform::Platform,
    LauncherError, Result,
};

/// Builds a download plan for the current platform.
///
/// The plan includes the client jar, libraries, and asset index. Asset objects
/// are planned after the index has been downloaded and read.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if required version metadata is missing or a
/// library coordinate is invalid.
pub fn plan_vanilla_downloads(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
) -> Result<DownloadPlan> {
    plan_vanilla_downloads_for_platform(
        version,
        minecraft_dir,
        Platform::current(),
        CompatibilityPolicy::Auto,
    )
}

/// Builds a vanilla download plan for an explicit platform and compatibility policy.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if required version metadata is missing or a
/// library coordinate is invalid.
pub fn plan_vanilla_downloads_for_platform(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
    platform: Platform,
    compatibility: CompatibilityPolicy,
) -> Result<DownloadPlan> {
    let compatibility = apply_compatibility(version, platform, compatibility);
    let version = &compatibility.version;
    let minecraft_dir = minecraft_dir.as_ref();
    let version_id = version
        .id
        .as_deref()
        .ok_or_else(|| LauncherError::MissingField {
            context: "version json".to_string(),
            field: "id".to_string(),
        })?;
    let jar_id = version.jar.as_deref().unwrap_or(version_id);

    let mut plan = DownloadPlan::default();
    if let Some(client) = version.downloads.get("client") {
        plan.tasks.push(DownloadTask {
            url: client.url.clone(),
            destination: minecraft_dir
                .join("versions")
                .join(jar_id)
                .join(format!("{jar_id}.jar")),
            checksum: Some(Checksum::Sha1(client.sha1.clone())),
            label: format!("client {jar_id}"),
        });
    }

    plan.tasks
        .extend(super::libraries::plan_library_downloads_for_platform(
            &version.libraries,
            minecraft_dir,
            platform,
        )?);
    plan.tasks.extend(super::assets::plan_asset_index_download(
        version,
        minecraft_dir,
    )?);
    Ok(plan)
}
