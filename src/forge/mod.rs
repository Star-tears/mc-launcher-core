//! Deprecated Forge compatibility wrappers.
//!
//! New code should use [`crate::loader::forge`] metadata helpers or install
//! Forge through [`crate::launcher::Launcher::install`].

use std::path::Path;

use crate::{loader::forge, Result};

/// Legacy wrapper for [`crate::loader::forge::list_forge_versions`].
#[deprecated(note = "use loader::forge::list_forge_versions")]
pub fn list_forge_versions() -> Result<Vec<String>> {
    forge::list_forge_versions()
}

/// Legacy wrapper for [`crate::loader::forge::forge_installed_version_id`].
#[deprecated(note = "use loader::forge::forge_installed_version_id")]
pub fn forge_to_installed_version(forge_version: &str) -> Result<String> {
    forge::forge_installed_version_id(forge_version)
}

/// Deprecated direct installer entry point.
///
/// Use [`crate::launcher::Launcher::install`] with
/// [`crate::loader::common::LoaderSpec::Forge`].
#[deprecated(note = "use loader::forge installer support through Launcher::install")]
pub fn run_forge_installer(version: &str, _java: Option<impl AsRef<Path>>) -> Result<()> {
    Err(crate::LauncherError::Other {
        message: format!(
            "direct Forge installer execution for {version} moved to Launcher::install"
        ),
    })
}
