use std::process::Command;

use mc_launcher_core::prelude::*;

fn main() -> mc_launcher_core::Result<()> {
    let minecraft_dir = std::env::current_dir()?.join(".minecraft");
    let launcher = Launcher::new(minecraft_dir);

    let install = launcher.install(InstallRequest {
        minecraft_version: "1.20.1".to_string(),
        loader: Some(LoaderSpec::Fabric {
            version: LoaderVersion::LatestStable,
        }),
        java: JavaInstallPolicy::Auto,
    })?;
    let version_json = launcher.load_version(&install.version_id)?;

    let command = launcher.build_launch_command_from_version(
        &version_json,
        LaunchOptions {
            account: Account::offline("Steve"),
            ..Default::default()
        },
    )?;

    let mut child = Command::new(&command.executable)
        .args(&command.args)
        .current_dir(&command.working_dir)
        .spawn()?;
    child.wait()?;
    Ok(())
}
