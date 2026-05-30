//! Native library extraction helpers.

use std::{
    collections::HashSet,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use zip::ZipArchive;

use crate::{
    core::{
        maven::MavenCoordinate,
        rules::{evaluate_rules, FeatureSet},
        version::{Library, LibraryArtifact},
    },
    io::paths::safe_join,
    platform::{Arch, Os, Platform},
    Result,
};

/// Returns the local native extraction directory for a version.
pub fn natives_directory(minecraft_dir: impl AsRef<Path>, version_id: &str) -> PathBuf {
    minecraft_dir
        .as_ref()
        .join("versions")
        .join(version_id)
        .join("natives")
}

/// Extracts native libraries for the current platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if a native jar cannot be read, a Maven
/// coordinate is invalid, or an archive entry would escape the destination.
pub fn extract_natives(
    libraries: &[Library],
    minecraft_dir: impl AsRef<Path>,
    version_id: &str,
) -> Result<PathBuf> {
    extract_natives_for_platform(libraries, minecraft_dir, version_id, Platform::current())
}

/// Extracts native libraries for an explicit platform.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if a native jar cannot be read, a Maven
/// coordinate is invalid, or an archive entry would escape the destination.
pub fn extract_natives_for_platform(
    libraries: &[Library],
    minecraft_dir: impl AsRef<Path>,
    version_id: &str,
    platform: Platform,
) -> Result<PathBuf> {
    let minecraft_dir = minecraft_dir.as_ref();
    let destination = natives_directory(minecraft_dir, version_id);
    fs::create_dir_all(&destination)?;

    let mut extracted = HashSet::new();
    for library in libraries {
        if !evaluate_rules(&library.rules, platform, &FeatureSet::default()) {
            continue;
        }

        for artifact in native_artifacts_for_platform(library, platform)? {
            if !extracted.insert(artifact.path.as_str()) {
                continue;
            }
            let jar_path = minecraft_dir.join("libraries").join(&artifact.path);
            extract_native_jar(&jar_path, &destination, library)?;
        }
    }

    Ok(destination)
}

fn native_artifacts_for_platform(
    library: &Library,
    platform: Platform,
) -> Result<Vec<&LibraryArtifact>> {
    let mut artifacts = Vec::new();
    let Some(downloads) = &library.downloads else {
        return Ok(artifacts);
    };

    if let Some(classifier) = native_classifier(library, platform) {
        if let Some(artifact) = downloads.classifiers.get(&classifier) {
            artifacts.push(artifact);
        }
    }

    let coordinate = MavenCoordinate::parse(&library.name)?;
    if native_coordinate_matches_platform(coordinate.classifier.as_deref(), platform) {
        if let Some(artifact) = &downloads.artifact {
            artifacts.push(artifact);
        }
    }

    Ok(artifacts)
}

fn extract_native_jar(jar_path: &Path, destination: &Path, library: &Library) -> Result<()> {
    let file = File::open(jar_path)?;
    let mut archive = ZipArchive::new(file)?;
    let excluded = library
        .extract
        .as_ref()
        .and_then(|extract| extract.get("exclude"))
        .cloned()
        .unwrap_or_default();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let entry_name = entry.name().to_string();
        if should_skip_entry(&entry_name, &excluded) {
            continue;
        }

        let Some(enclosed) = entry.enclosed_name().map(|path| path.to_path_buf()) else {
            safe_join(destination, &entry_name)?;
            continue;
        };
        let output = safe_join(destination, enclosed)?;
        if entry.is_dir() {
            fs::create_dir_all(&output)?;
            continue;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = File::create(output)?;
        io::copy(&mut entry, &mut output_file)?;
    }

    Ok(())
}

fn should_skip_entry(name: &str, excluded: &[String]) -> bool {
    name.starts_with("META-INF/") || excluded.iter().any(|prefix| name.starts_with(prefix))
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

fn native_coordinate_matches_platform(classifier: Option<&str>, platform: Platform) -> bool {
    let Some(classifier) = classifier else {
        return false;
    };

    match platform.os {
        Os::MacOs => match platform.arch {
            Arch::Aarch64 => matches!(
                classifier,
                "natives-macos-arm64" | "natives-osx-arm64" | "natives-macos"
            ),
            Arch::X86 => matches!(classifier, "natives-macos-x86" | "natives-osx-x86"),
            _ => matches!(classifier, "natives-macos" | "natives-osx"),
        },
        Os::Windows => match platform.arch {
            Arch::Aarch64 => classifier == "natives-windows-arm64",
            Arch::X86 => classifier == "natives-windows-x86",
            _ => classifier == "natives-windows" || classifier == "natives-windows-x86_64",
        },
        Os::Linux => match platform.arch {
            Arch::Aarch64 => classifier == "natives-linux-arm64",
            Arch::X86 => classifier == "natives-linux-x86",
            _ => classifier == "natives-linux" || classifier == "natives-linux-x86_64",
        },
        Os::Other => false,
    }
}
