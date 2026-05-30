pub mod assets;
pub mod client;
pub mod libraries;
pub mod loader;
pub mod natives;
pub mod request;
pub mod vanilla;

use std::path::Path;

pub use request::{InstallRequest, InstallResult, JavaInstallPolicy};

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
