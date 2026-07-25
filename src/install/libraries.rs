//! Library download planning.

use std::path::Path;

use crate::{
    core::{
        maven::MavenCoordinate,
        rules::{evaluate_rules, FeatureSet},
        version::{Library, LibraryArtifact},
    },
    net::download::{Checksum, DownloadTask},
    platform::Platform,
    Result,
};

/// Plans library and native-classifier downloads for the current platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if a library coordinate cannot be parsed.
pub fn plan_library_downloads(
    libraries: &[Library],
    minecraft_dir: &Path,
) -> Result<Vec<DownloadTask>> {
    plan_library_downloads_for_platform(libraries, minecraft_dir, Platform::current())
}

/// Plans library and native-classifier downloads for an explicit platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if a library coordinate cannot be parsed.
pub fn plan_library_downloads_for_platform(
    libraries: &[Library],
    minecraft_dir: &Path,
    platform: Platform,
) -> Result<Vec<DownloadTask>> {
    let mut tasks = Vec::new();
    for library in libraries {
        if !evaluate_rules(&library.rules, platform, &FeatureSet::default()) {
            continue;
        }
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                tasks.push(download_task(library, artifact, minecraft_dir));
            }
            if let Some(classifier) = native_classifier(library, platform) {
                if let Some(artifact) = downloads.classifiers.get(&classifier) {
                    tasks.push(download_task(library, artifact, minecraft_dir));
                }
            }
        } else if library.natives.is_none() {
            // Fabric/Quilt-style libraries: no `downloads` block, just a
            // Maven coordinate + repo base URL. Mirrors the fallback in
            // classpath.rs — build the artifact path and repo URL manually.
            let coordinate = MavenCoordinate::parse(&library.name)?;
            let path = coordinate.artifact_path();
            let base_url = library
                .url
                .clone()
                .unwrap_or_else(|| "https://maven.fabricmc.net/".to_string());
            let url = format!(
                "{}/{}",
                base_url.trim_end_matches('/'),
                path.to_string_lossy().replace('\\', "/")
            );
            tasks.push(DownloadTask {
                url,
                destination: minecraft_dir.join("libraries").join(&path),
                checksum: None,
                label: format!("library {}", library.name),
            });
        }
    }
    Ok(tasks)
}

fn download_task(
    library: &Library,
    artifact: &LibraryArtifact,
    minecraft_dir: &Path,
) -> DownloadTask {
    DownloadTask {
        url: artifact.url.clone(),
        destination: minecraft_dir.join("libraries").join(&artifact.path),
        checksum: Some(Checksum::Sha1(artifact.sha1.clone())),
        label: library.name.clone(),
    }
}

fn native_classifier(library: &Library, platform: Platform) -> Option<String> {
    library
        .natives
        .as_ref()?
        .get(platform.minecraft_os_name())
        .map(|classifier| {
            classifier.replace("${arch}", if platform.is_32_bit() { "32" } else { "64" })
        })
}
