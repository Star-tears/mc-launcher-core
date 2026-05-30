//! Installation planning and execution.
//!
//! The high-level [`crate::launcher::Launcher`] facade uses this module to fetch
//! version metadata, write profiles, plan downloads, install assets, and extract
//! native libraries. Advanced launchers can call the planning functions directly
//! to preview or customize install work.

pub mod assets;
pub mod client;
pub mod libraries;
pub mod loader;
pub mod natives;
pub mod request;
pub mod vanilla;

use std::path::Path;

pub use request::{InstallRequest, InstallResult, JavaInstallPolicy};

/// Legacy vanilla install wrapper.
///
/// Prefer [`crate::launcher::Launcher::install`], which supports loader
/// profiles and returns the installed profile id.
///
/// # Errors
///
/// Returns an error when version metadata, downloads, or filesystem writes fail.
#[deprecated(note = "use Launcher::install")]
pub fn install_minecraft_version(
    version_id: &str,
    minecraft_directory: impl AsRef<Path>,
    _callback: &crate::types::CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    let launcher = crate::launcher::Launcher::new(minecraft_directory.as_ref().to_path_buf());
    launcher.install(InstallRequest::vanilla(version_id))?;
    Ok(())
}
