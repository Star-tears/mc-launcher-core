//! Launch command construction.
//!
//! This module converts merged Minecraft version metadata into a Java
//! executable, argument list, working directory, and environment block. It does
//! not spawn the process; callers can inspect or adjust the returned
//! [`LaunchCommand`] before passing it to [`std::process::Command`].

use std::path::PathBuf;

use crate::{
    account::Account,
    compatibility::{apply_compatibility, CompatibilityPolicy},
    core::{
        arguments::{evaluate_arguments, ArgumentContext},
        classpath::{classpath_entries_for_platform, classpath_string},
        rules::FeatureSet,
        version::VersionJson,
    },
    platform::{Os, Platform},
    LauncherError, Result,
};

/// User and process settings used while building a launch command.
///
/// Start with [`LaunchOptions::default`] and override only the fields your
/// launcher exposes. By default the game directory is isolated under
/// `<minecraft_dir>/versions/<version_id>`, which keeps saves, options, logs,
/// and mods separate per installed profile.
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    /// Account values substituted into Minecraft's auth placeholders.
    pub account: Account,
    /// Java executable to run.
    ///
    /// If omitted, the command uses `java` and relies on the caller's `PATH`.
    pub java_executable: Option<PathBuf>,
    /// Game directory passed as `${game_directory}` and used as process CWD.
    ///
    /// If omitted, a version-isolated directory is used.
    pub game_directory: Option<PathBuf>,
    /// Directory containing extracted native libraries.
    ///
    /// If omitted, this points at `<minecraft_dir>/versions/<version_id>/natives`.
    pub natives_directory: Option<PathBuf>,
    /// Launcher name passed to modern version argument templates.
    pub launcher_name: String,
    /// Launcher version passed to modern version argument templates.
    pub launcher_version: String,
    /// Optional window size appended as `--width` and `--height`.
    pub custom_resolution: Option<(u32, u32)>,
    /// Enables Minecraft demo mode.
    pub demo: bool,
    /// Optional multiplayer server and port to join after launch.
    pub server: Option<(String, Option<u16>)>,
    /// Appends the modern `--disableMultiplayer` flag.
    pub disable_multiplayer: bool,
    /// Appends the modern `--disableChat` flag.
    pub disable_chat: bool,
    /// Controls whether known compatibility patches are applied before building.
    pub compatibility: CompatibilityPolicy,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            account: Account::offline("Steve"),
            java_executable: None,
            game_directory: None,
            natives_directory: None,
            launcher_name: "mc-launcher-core".to_string(),
            launcher_version: env!("CARGO_PKG_VERSION").to_string(),
            custom_resolution: None,
            demo: false,
            server: None,
            disable_multiplayer: false,
            disable_chat: false,
            compatibility: CompatibilityPolicy::Auto,
        }
    }
}

/// A Java process description ready to spawn.
///
/// The command is intentionally returned as structured parts instead of a shell
/// string so launchers can avoid quoting bugs across Windows, macOS, and Linux.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCommand {
    /// Java executable path or command name.
    pub executable: PathBuf,
    /// JVM, main class, and game arguments in process order.
    pub args: Vec<String>,
    /// Directory that should be used as the child process current directory.
    pub working_dir: PathBuf,
    /// Environment variables to set on the child process.
    pub env: Vec<(String, String)>,
}

impl LaunchCommand {
    /// Returns the executable and argument list for callers that do not need
    /// working-directory or environment metadata.
    pub fn to_process_parts(&self) -> (PathBuf, Vec<String>) {
        (self.executable.clone(), self.args.clone())
    }
}

/// Builds a launch command for the current platform.
///
/// This is the lower-level equivalent of
/// [`crate::launcher::Launcher::build_launch_command_from_version`].
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if required version fields are missing or if
/// library coordinates cannot be converted into classpath entries.
pub fn build_launch_command(
    version: &VersionJson,
    minecraft_dir: PathBuf,
    options: LaunchOptions,
) -> Result<LaunchCommand> {
    build_launch_command_for_platform(version, minecraft_dir, options, Platform::current())
}

/// Builds a launch command for an explicit platform.
///
/// This function is mainly useful for tests, planning tools, or launchers that
/// need to inspect cross-platform output. Normal applications should call
/// [`build_launch_command`].
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the version metadata cannot produce a
/// complete executable command.
pub fn build_launch_command_for_platform(
    version: &VersionJson,
    minecraft_dir: PathBuf,
    options: LaunchOptions,
    platform: Platform,
) -> Result<LaunchCommand> {
    let compatibility = apply_compatibility(version, platform, options.compatibility);
    let version = &compatibility.version;
    let version_id = version
        .id
        .as_deref()
        .ok_or_else(|| LauncherError::MissingField {
            context: "version json".to_string(),
            field: "id".to_string(),
        })?;
    let main_class = version
        .main_class
        .clone()
        .ok_or_else(|| LauncherError::MissingField {
            context: version_id.to_string(),
            field: "mainClass".to_string(),
        })?;

    let game_dir = options
        .game_directory
        .clone()
        .unwrap_or_else(|| minecraft_dir.join("versions").join(version_id));
    let natives_dir = options.natives_directory.clone().unwrap_or_else(|| {
        minecraft_dir
            .join("versions")
            .join(version_id)
            .join("natives")
    });
    let entries = classpath_entries_for_platform(version, &minecraft_dir, platform)?;
    let classpath = classpath_string(&entries);
    let assets_index = version.assets.as_deref().unwrap_or(version_id);
    let version_type = version.r#type.as_deref().unwrap_or("release");

    let features = FeatureSet {
        demo_user: options.demo,
        custom_resolution: options.custom_resolution.is_some(),
        ..Default::default()
    };
    let context = ArgumentContext {
        minecraft_dir: &minecraft_dir,
        natives_dir: &natives_dir,
        game_dir: &game_dir,
        version,
        account: &options.account,
        classpath: &classpath,
        launcher_name: &options.launcher_name,
        launcher_version: &options.launcher_version,
        version_type,
        assets_index,
        extra: Default::default(),
    };

    let executable = options
        .java_executable
        .unwrap_or_else(|| PathBuf::from("java"));
    let mut args = evaluate_arguments(&version.arguments.jvm, &context, &features, platform);
    if args.is_empty() {
        args.extend(default_legacy_jvm_arguments(
            &natives_dir,
            &classpath,
            platform,
        ));
    }
    args.push(main_class);

    if version.minecraft_arguments.is_some() {
        let legacy = version
            .minecraft_arguments
            .as_deref()
            .unwrap_or_default()
            .split(' ')
            .map(|part| crate::core::arguments::replace_placeholders(part, &context));
        args.extend(legacy);
    } else {
        args.extend(evaluate_arguments(
            &version.arguments.game,
            &context,
            &features,
            platform,
        ));
    }

    if let Some((width, height)) = options.custom_resolution {
        args.extend([
            "--width".to_string(),
            width.to_string(),
            "--height".to_string(),
            height.to_string(),
        ]);
    }
    if options.demo {
        args.push("--demo".to_string());
    }
    if let Some((server, port)) = options.server {
        args.extend(["--server".to_string(), server]);
        if let Some(port) = port {
            args.extend(["--port".to_string(), port.to_string()]);
        }
    }
    if options.disable_multiplayer {
        args.push("--disableMultiplayer".to_string());
    }
    if options.disable_chat {
        args.push("--disableChat".to_string());
    }

    Ok(LaunchCommand {
        executable,
        args,
        working_dir: game_dir,
        env: Vec::new(),
    })
}

fn default_legacy_jvm_arguments(
    natives_dir: &std::path::Path,
    classpath: &str,
    platform: Platform,
) -> Vec<String> {
    let mut args = Vec::new();
    if platform.os == Os::MacOs {
        args.push("-XstartOnFirstThread".to_string());
    }
    args.push(format!("-Djava.library.path={}", natives_dir.display()));
    args.push("-cp".to_string());
    args.push(classpath.to_string());
    args
}
