use std::path::{Path, PathBuf};

use crate::{
    command::builder::{build_launch_command, LaunchCommand, LaunchOptions},
    core::version::VersionJson,
    install::request::{InstallRequest, InstallResult},
    Result,
};

#[derive(Debug, Clone)]
pub struct Launcher {
    minecraft_dir: PathBuf,
}

impl Launcher {
    pub fn new(minecraft_dir: impl Into<PathBuf>) -> Self {
        Self {
            minecraft_dir: minecraft_dir.into(),
        }
    }

    pub fn minecraft_dir(&self) -> &Path {
        &self.minecraft_dir
    }

    pub fn install(&self, request: InstallRequest) -> Result<InstallResult> {
        Ok(InstallResult {
            version_id: request.minecraft_version,
        })
    }

    pub fn build_launch_command_from_version(
        &self,
        version: &VersionJson,
        options: LaunchOptions,
    ) -> Result<LaunchCommand> {
        build_launch_command(version, self.minecraft_dir.clone(), options)
    }
}
