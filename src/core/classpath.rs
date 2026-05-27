use std::path::{Path, PathBuf};

use crate::{
    core::{maven::MavenCoordinate, version::VersionJson},
    Result,
};

pub fn classpath_entries(
    version: &VersionJson,
    minecraft_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>> {
    let minecraft_dir = minecraft_dir.as_ref();
    let mut entries = Vec::new();

    for library in &version.libraries {
        let coordinate = MavenCoordinate::parse(&library.name)?;
        entries.push(
            minecraft_dir
                .join("libraries")
                .join(coordinate.artifact_path()),
        );
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

pub fn classpath_string(entries: &[PathBuf]) -> String {
    let separator = super::arguments::classpath_separator();
    entries
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(separator)
}
