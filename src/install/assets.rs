use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    core::version::VersionJson,
    net::download::{execute_plan, Checksum, DownloadPlan, DownloadTask},
    progress::ProgressReporter,
    Result,
};

const RESOURCES_BASE: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct AssetIndexJson {
    #[serde(default)]
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssetObject {
    pub hash: String,
    pub size: i64,
}

pub fn asset_index_path(minecraft_dir: impl AsRef<Path>, asset_index_id: &str) -> PathBuf {
    minecraft_dir
        .as_ref()
        .join("assets")
        .join("indexes")
        .join(format!("{asset_index_id}.json"))
}

pub fn asset_object_path(minecraft_dir: impl AsRef<Path>, hash: &str) -> PathBuf {
    let prefix = hash.get(..2).unwrap_or(hash);
    minecraft_dir
        .as_ref()
        .join("assets")
        .join("objects")
        .join(prefix)
        .join(hash)
}

pub fn plan_asset_index_download(
    version: &VersionJson,
    minecraft_dir: &Path,
) -> Result<Vec<DownloadTask>> {
    let Some(asset_index) = &version.asset_index else {
        return Ok(Vec::new());
    };
    Ok(vec![DownloadTask {
        url: asset_index.url.clone(),
        destination: asset_index_path(minecraft_dir, &asset_index.id),
        checksum: Some(Checksum::Sha1(asset_index.sha1.clone())),
        label: format!("assets index {}", asset_index.id),
    }])
}

pub fn plan_asset_object_downloads_from_index(
    index: &AssetIndexJson,
    minecraft_dir: impl AsRef<Path>,
) -> DownloadPlan {
    let minecraft_dir = minecraft_dir.as_ref();
    let mut objects = index.objects.iter().collect::<Vec<_>>();
    objects.sort_by_key(|(name, _)| name.as_str());

    DownloadPlan {
        tasks: objects
            .into_iter()
            .map(|(name, object)| {
                let prefix = object.hash.get(..2).unwrap_or(&object.hash);
                DownloadTask {
                    url: format!("{RESOURCES_BASE}/{prefix}/{}", object.hash),
                    destination: asset_object_path(minecraft_dir, &object.hash),
                    checksum: Some(Checksum::Sha1(object.hash.clone())),
                    label: format!("asset {name}"),
                }
            })
            .collect(),
    }
}

pub fn install_assets(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
    reporter: &mut dyn ProgressReporter,
) -> Result<()> {
    let minecraft_dir = minecraft_dir.as_ref();
    let index_plan = DownloadPlan {
        tasks: plan_asset_index_download(version, minecraft_dir)?,
    };
    execute_plan(&index_plan, reporter)?;

    let Some(asset_index) = &version.asset_index else {
        return Ok(());
    };
    let index_path = asset_index_path(minecraft_dir, &asset_index.id);
    let index: AssetIndexJson = serde_json::from_slice(&fs::read(index_path)?)?;
    let object_plan = plan_asset_object_downloads_from_index(&index, minecraft_dir);
    execute_plan(&object_plan, reporter)
}
