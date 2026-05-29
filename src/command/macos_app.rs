use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{command::builder::LaunchCommand, Result};

/// Metadata used when generating a macOS `.app` host bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacOsAppBundleOptions {
    pub bundle_name: String,
    pub bundle_identifier: String,
    /// Optional file that receives stdout from the app process via `open --stdout`.
    pub stdout_path: Option<PathBuf>,
    /// Optional file that receives stderr from the app process via `open --stderr`.
    pub stderr_path: Option<PathBuf>,
}

impl Default for MacOsAppBundleOptions {
    fn default() -> Self {
        Self {
            bundle_name: "Minecraft Legacy Host".to_string(),
            bundle_identifier: "dev.mc-launcher-core.legacy-host".to_string(),
            stdout_path: None,
            stderr_path: None,
        }
    }
}

/// A generated macOS app bundle plus the `open` command used to launch it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacOsAppBundle {
    pub bundle_path: PathBuf,
    pub open_command: LaunchCommand,
}

/// Write a macOS `.app` bundle that hosts an existing Java launch command.
pub fn prepare_macos_app_bundle(
    command: &LaunchCommand,
    bundle_path: impl AsRef<Path>,
    options: MacOsAppBundleOptions,
) -> Result<MacOsAppBundle> {
    let bundle_path = bundle_path.as_ref().to_path_buf();
    let contents_dir = bundle_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    fs::create_dir_all(&macos_dir)?;

    fs::write(contents_dir.join("Info.plist"), info_plist(&options))?;
    let launcher_path = macos_dir.join("launch");
    fs::write(&launcher_path, launch_script(command))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&launcher_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&launcher_path, permissions)?;
    }

    Ok(MacOsAppBundle {
        bundle_path: bundle_path.clone(),
        open_command: LaunchCommand {
            executable: PathBuf::from("/usr/bin/open"),
            args: open_command_args(&bundle_path, &options),
            working_dir: command.working_dir.clone(),
            env: Vec::new(),
        },
    })
}

fn open_command_args(bundle_path: &Path, options: &MacOsAppBundleOptions) -> Vec<String> {
    let mut args = vec!["-W".to_string(), "-n".to_string()];
    if let Some(path) = &options.stdout_path {
        args.extend(["--stdout".to_string(), path.to_string_lossy().to_string()]);
    }
    if let Some(path) = &options.stderr_path {
        args.extend(["--stderr".to_string(), path.to_string_lossy().to_string()]);
    }
    args.push(bundle_path.to_string_lossy().to_string());
    args
}

fn info_plist(options: &MacOsAppBundleOptions) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>launch</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"#,
        xml_escape(&options.bundle_identifier),
        xml_escape(&options.bundle_name)
    )
}

fn launch_script(command: &LaunchCommand) -> String {
    let mut exec_parts = vec!["exec".to_string(), "env".to_string()];
    exec_parts.extend(
        command
            .env
            .iter()
            .map(|(key, value)| shell_quote(&format!("{key}={value}"))),
    );
    exec_parts.push(shell_quote_path(&command.executable));
    exec_parts.extend(command.args.iter().map(|arg| shell_quote(arg)));

    format!(
        "#!/bin/sh\nset -eu\ncd {}\n{}\n",
        shell_quote_path(&command.working_dir),
        exec_parts.join(" ")
    )
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.to_string_lossy())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
