use std::path::Path;

use crate::{
    core::version::VersionJson,
    net::download::{Checksum, DownloadTask},
    Result,
};

pub fn plan_asset_index_download(
    version: &VersionJson,
    minecraft_dir: &Path,
) -> Result<Vec<DownloadTask>> {
    let Some(asset_index) = &version.asset_index else {
        return Ok(Vec::new());
    };
    Ok(vec![DownloadTask {
        url: asset_index.url.clone(),
        destination: minecraft_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", asset_index.id)),
        checksum: Some(Checksum::Sha1(asset_index.sha1.clone())),
        label: format!("assets index {}", asset_index.id),
    }])
}
