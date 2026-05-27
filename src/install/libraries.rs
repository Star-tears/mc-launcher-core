use std::path::Path;

use crate::{
    core::version::Library,
    net::download::{Checksum, DownloadTask},
    Result,
};

pub fn plan_library_downloads(
    libraries: &[Library],
    minecraft_dir: &Path,
) -> Result<Vec<DownloadTask>> {
    let mut tasks = Vec::new();
    for library in libraries {
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                tasks.push(DownloadTask {
                    url: artifact.url.clone(),
                    destination: minecraft_dir.join("libraries").join(&artifact.path),
                    checksum: Some(Checksum::Sha1(artifact.sha1.clone())),
                    label: library.name.clone(),
                });
            }
        }
    }
    Ok(tasks)
}
