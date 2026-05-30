//! Command builders and legacy command helpers.
//!
//! New code should prefer [`builder::build_launch_command`] or the higher-level
//! [`crate::launcher::Launcher::build_launch_command_from_version`] facade.

pub mod builder;

use std::{fs, path::Path};

use crate::{core::version::VersionJson, types::MinecraftOptions, LauncherError};

/// Legacy helper that returns a flattened Java command vector.
///
/// Prefer [`crate::launcher::Launcher::build_launch_command_from_version`],
/// which returns a structured [`builder::LaunchCommand`].
///
/// # Errors
///
/// Returns an error if the version JSON is missing, cannot be parsed, or cannot
/// be converted into a launch command.
#[deprecated(note = "use Launcher::build_launch_command_from_version")]
pub fn get_minecraft_command(
    version: &str,
    minecraft_directory: impl AsRef<Path>,
    _options_arg: &MinecraftOptions,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = minecraft_directory
        .as_ref()
        .join("versions")
        .join(version)
        .join(format!("{version}.json"));
    if !path.is_file() {
        return Err(Box::new(LauncherError::InvalidVersionId {
            id: version.to_string(),
        }));
    }
    let version_json: VersionJson = serde_json::from_str(&fs::read_to_string(path)?)?;
    let command = builder::build_launch_command(
        &version_json,
        minecraft_directory.as_ref().to_path_buf(),
        Default::default(),
    )?;
    let mut parts = vec![command.executable.to_string_lossy().to_string()];
    parts.extend(command.args);
    Ok(parts)
}
