use std::path::Path;

use crate::{
    core::version::VersionJson,
    net::download::{Checksum, DownloadPlan, DownloadTask},
    LauncherError, Result,
};

pub fn plan_vanilla_downloads(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
) -> Result<DownloadPlan> {
    let minecraft_dir = minecraft_dir.as_ref();
    let version_id = version
        .id
        .as_deref()
        .ok_or_else(|| LauncherError::MissingField {
            context: "version json".to_string(),
            field: "id".to_string(),
        })?;

    let mut plan = DownloadPlan::default();
    if let Some(client) = version.downloads.get("client") {
        plan.tasks.push(DownloadTask {
            url: client.url.clone(),
            destination: minecraft_dir
                .join("versions")
                .join(version_id)
                .join(format!("{version_id}.jar")),
            checksum: Some(Checksum::Sha1(client.sha1.clone())),
            label: format!("client {version_id}"),
        });
    }

    plan.tasks.extend(super::libraries::plan_library_downloads(
        &version.libraries,
        minecraft_dir,
    )?);
    plan.tasks.extend(super::assets::plan_asset_index_download(
        version,
        minecraft_dir,
    )?);
    Ok(plan)
}
