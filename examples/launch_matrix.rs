use std::process::Command;

use mc_launcher_core::prelude::*;

const VERSIONS: &[&str] = &["1.16.5", "1.18.2", "1.20.1", "26.1.2"];

fn main() -> mc_launcher_core::Result<()> {
    let minecraft_dir = std::env::current_dir()?.join(".minecraft");
    let launcher = Launcher::new(minecraft_dir);

    for minecraft_version in VERSIONS {
        launch(&launcher, InstallRequest::vanilla(*minecraft_version))?;
        launch(
            &launcher,
            InstallRequest {
                minecraft_version: (*minecraft_version).to_string(),
                loader: Some(LoaderSpec::Fabric {
                    version: LoaderVersion::LatestStable,
                }),
                java: JavaInstallPolicy::Auto,
            },
        )?;
    }

    Ok(())
}

fn launch(launcher: &Launcher, request: InstallRequest) -> mc_launcher_core::Result<()> {
    let install = launcher.install(request)?;
    let version = launcher.load_version(&install.version_id)?;
    let command = launcher.build_launch_command_from_version(
        &version,
        LaunchOptions {
            account: Account::offline("Steve"),
            ..Default::default()
        },
    )?;

    let status = Command::new(&command.executable)
        .args(&command.args)
        .current_dir(&command.working_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(LauncherError::Other {
            message: format!("{} exited with {status}", install.version_id),
        })
    }
}
