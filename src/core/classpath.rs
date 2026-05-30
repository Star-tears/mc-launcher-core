//! Classpath construction from version metadata.

use std::path::{Path, PathBuf};

use crate::{
    core::{
        maven::MavenCoordinate,
        rules::{evaluate_rules, FeatureSet},
        version::VersionJson,
    },
    platform::Platform,
    Result,
};

/// Returns classpath entries for the current platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if a fallback Maven coordinate cannot be
/// parsed.
pub fn classpath_entries(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>> {
    classpath_entries_for_platform(version, minecraft_dir, Platform::current())
}

/// Returns classpath entries for an explicit platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if a fallback Maven coordinate cannot be
/// parsed.
pub fn classpath_entries_for_platform(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
    platform: Platform,
) -> Result<Vec<PathBuf>> {
    let minecraft_dir = minecraft_dir.as_ref();
    let mut entries = Vec::new();

    for library in &version.libraries {
        if !evaluate_rules(&library.rules, platform, &FeatureSet::default()) {
            continue;
        }
        if let Some(artifact) = library
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.artifact.as_ref())
        {
            entries.push(minecraft_dir.join("libraries").join(&artifact.path));
        } else if library.natives.is_none() {
            let coordinate = MavenCoordinate::parse(&library.name)?;
            entries.push(
                minecraft_dir
                    .join("libraries")
                    .join(coordinate.artifact_path()),
            );
        }
    }

    let jar_id = version.jar.as_ref().or(version.id.as_ref());
    if let Some(id) = jar_id {
        entries.push(
            minecraft_dir
                .join("versions")
                .join(id)
                .join(format!("{id}.jar")),
        );
    }

    Ok(entries)
}

/// Joins classpath entries with the platform separator.
pub fn classpath_string(entries: &[PathBuf]) -> String {
    let separator = super::arguments::classpath_separator();
    entries
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(separator)
}
