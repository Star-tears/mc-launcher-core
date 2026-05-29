use std::path::{Path, PathBuf};

use crate::{
    command::{
        builder::{build_launch_command, LaunchCommand, LaunchOptions},
        macos_app::{prepare_macos_app_bundle, MacOsAppBundle, MacOsAppBundleOptions},
    },
    core::version::VersionJson,
    install::{
        loader::{run_loader_installer, write_loader_profile, InstallerInvocation},
        request::{InstallRequest, InstallResult},
    },
    loader::{
        common::{LoaderSpec, LoaderVersion},
        LoaderKind,
    },
    net::download::{execute_plan, DownloadPlan, DownloadTask},
    progress::ProgressEvent,
    LauncherError, Result,
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
        if let Some(loader) = request.loader {
            match loader {
                LoaderSpec::Fabric { version } => {
                    let loader_version = resolve_fabric_loader_version(version)?;
                    let profile = crate::loader::fabric::fetch_profile(
                        &request.minecraft_version,
                        &loader_version,
                    )?;
                    write_loader_profile(&self.minecraft_dir, &profile)?;
                    let version_id = profile.id.ok_or_else(|| LauncherError::MissingField {
                        context: "loader profile".to_string(),
                        field: "id".to_string(),
                    })?;
                    return Ok(InstallResult { version_id });
                }
                LoaderSpec::Quilt { version } => {
                    let loader_version = resolve_quilt_loader_version(version)?;
                    let profile = crate::loader::quilt::fetch_profile(
                        &request.minecraft_version,
                        &loader_version,
                    )?;
                    write_loader_profile(&self.minecraft_dir, &profile)?;
                    let version_id = profile.id.ok_or_else(|| LauncherError::MissingField {
                        context: "loader profile".to_string(),
                        field: "id".to_string(),
                    })?;
                    return Ok(InstallResult { version_id });
                }
                LoaderSpec::Forge { version } => {
                    let loader_version = resolve_forge_loader_version(version)?;
                    let installer_path = download_installer(
                        &self.minecraft_dir,
                        "forge",
                        &loader_version,
                        &crate::loader::forge::installer_url(&loader_version),
                    )?;
                    run_loader_installer(&InstallerInvocation {
                        loader: LoaderKind::Forge,
                        java_executable: PathBuf::from("java"),
                        installer_path,
                        minecraft_dir: self.minecraft_dir.clone(),
                    })?;
                    return Ok(InstallResult {
                        version_id: crate::loader::forge::forge_installed_version_id(
                            &loader_version,
                        )?,
                    });
                }
                LoaderSpec::NeoForge { version } => {
                    let loader_version = resolve_neoforge_loader_version(version)?;
                    let installer_path = download_installer(
                        &self.minecraft_dir,
                        "neoforge",
                        &loader_version,
                        &crate::loader::neoforge::installer_url(&loader_version),
                    )?;
                    run_loader_installer(&InstallerInvocation {
                        loader: LoaderKind::NeoForge,
                        java_executable: PathBuf::from("java"),
                        installer_path,
                        minecraft_dir: self.minecraft_dir.clone(),
                    })?;
                    return Ok(InstallResult {
                        version_id: crate::loader::neoforge::neoforge_installed_version_id(
                            &request.minecraft_version,
                            &loader_version,
                        ),
                    });
                }
            }
        }

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

    /// Prepare a macOS `.app` host bundle under this launcher's Minecraft directory.
    pub fn prepare_macos_app_bundle_launch(
        &self,
        command: &LaunchCommand,
        options: MacOsAppBundleOptions,
    ) -> Result<MacOsAppBundle> {
        let bundle_path = self
            .minecraft_dir
            .join("launcher-hosts")
            .join("macos")
            .join(format!(
                "{}.app",
                macos_bundle_directory_name(&options.bundle_name)
            ));
        prepare_macos_app_bundle(command, bundle_path, options)
    }
}

fn macos_bundle_directory_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect::<String>()
        .trim()
        .to_string();

    if sanitized.is_empty() {
        MacOsAppBundleOptions::default().bundle_name
    } else {
        sanitized
    }
}

fn resolve_fabric_loader_version(version: LoaderVersion) -> Result<String> {
    match version {
        LoaderVersion::Exact(version) => Ok(version),
        LoaderVersion::Latest | LoaderVersion::LatestStable => {
            let versions = crate::loader::fabric::list_loader_versions()?;
            Ok(crate::loader::fabric::latest_stable_loader(&versions)?
                .version
                .clone())
        }
    }
}

fn resolve_quilt_loader_version(version: LoaderVersion) -> Result<String> {
    match version {
        LoaderVersion::Exact(version) => Ok(version),
        LoaderVersion::Latest | LoaderVersion::LatestStable => {
            let versions = crate::loader::quilt::list_loader_versions()?;
            Ok(crate::loader::quilt::latest_loader(&versions)?
                .version
                .clone())
        }
    }
}

fn resolve_forge_loader_version(version: LoaderVersion) -> Result<String> {
    match version {
        LoaderVersion::Exact(version) => Ok(version),
        LoaderVersion::Latest | LoaderVersion::LatestStable => {
            let versions = crate::loader::forge::list_forge_versions()?;
            versions
                .last()
                .cloned()
                .ok_or_else(|| LauncherError::LoaderVersionNotFound {
                    loader: LoaderKind::Forge,
                    version: "latest".to_string(),
                })
        }
    }
}

fn resolve_neoforge_loader_version(version: LoaderVersion) -> Result<String> {
    match version {
        LoaderVersion::Exact(version) => Ok(version),
        LoaderVersion::Latest | LoaderVersion::LatestStable => {
            let versions = crate::loader::neoforge::list_neoforge_versions()?;
            versions
                .last()
                .cloned()
                .ok_or_else(|| LauncherError::LoaderVersionNotFound {
                    loader: LoaderKind::NeoForge,
                    version: "latest".to_string(),
                })
        }
    }
}

fn download_installer(
    minecraft_dir: &Path,
    loader_name: &str,
    loader_version: &str,
    url: &str,
) -> Result<PathBuf> {
    let destination = minecraft_dir
        .join("versions")
        .join(".installers")
        .join(format!("{loader_name}-{loader_version}-installer.jar"));
    let plan = DownloadPlan {
        tasks: vec![DownloadTask {
            url: url.to_string(),
            destination: destination.clone(),
            checksum: None,
            label: format!("{loader_name} installer {loader_version}"),
        }],
    };
    let mut reporter = |_event: ProgressEvent| {};
    execute_plan(&plan, &mut reporter)?;
    Ok(destination)
}
