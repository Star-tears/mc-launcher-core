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

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub account: Account,
    pub java_executable: Option<PathBuf>,
    pub game_directory: Option<PathBuf>,
    pub natives_directory: Option<PathBuf>,
    pub launcher_name: String,
    pub launcher_version: String,
    pub custom_resolution: Option<(u32, u32)>,
    pub demo: bool,
    pub server: Option<(String, Option<u16>)>,
    pub disable_multiplayer: bool,
    pub disable_chat: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub env: Vec<(String, String)>,
}

impl LaunchCommand {
    pub fn to_process_parts(&self) -> (PathBuf, Vec<String>) {
        (self.executable.clone(), self.args.clone())
    }
}

pub fn build_launch_command(
    version: &VersionJson,
    minecraft_dir: PathBuf,
    options: LaunchOptions,
) -> Result<LaunchCommand> {
    build_launch_command_for_platform(version, minecraft_dir, options, Platform::current())
}

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
        .unwrap_or_else(|| minecraft_dir.clone());
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
