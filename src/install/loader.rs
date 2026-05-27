use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{core::version::VersionJson, loader::LoaderKind, LauncherError, Result};

pub fn loader_profile_path(minecraft_dir: impl AsRef<Path>, version_id: &str) -> PathBuf {
    minecraft_dir
        .as_ref()
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"))
}

pub fn write_loader_profile(
    minecraft_dir: impl AsRef<Path>,
    profile: &VersionJson,
) -> Result<PathBuf> {
    let version_id = profile
        .id
        .as_deref()
        .ok_or_else(|| LauncherError::MissingField {
            context: "loader profile".to_string(),
            field: "id".to_string(),
        })?;
    let path = loader_profile_path(minecraft_dir, version_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_vec_pretty(profile)?)?;
    Ok(path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerInvocation {
    pub loader: LoaderKind,
    pub java_executable: PathBuf,
    pub installer_path: PathBuf,
    pub minecraft_dir: PathBuf,
}

pub fn installer_command_args(invocation: &InstallerInvocation) -> Vec<String> {
    vec![
        "-jar".to_string(),
        invocation.installer_path.to_string_lossy().to_string(),
        "--installClient".to_string(),
        invocation.minecraft_dir.to_string_lossy().to_string(),
    ]
}

pub fn run_loader_installer(invocation: &InstallerInvocation) -> Result<()> {
    let status = Command::new(&invocation.java_executable)
        .args(installer_command_args(invocation))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(LauncherError::InstallerFailed {
            loader: invocation.loader,
            status: status.code(),
        })
    }
}
