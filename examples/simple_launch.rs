use std::process::Command;

use mc_launcher_core::prelude::*;

fn main() -> mc_launcher_core::Result<()> {
    let minecraft_dir = mc_launcher_core::utils::get_minecraft_directory();
    let launcher = Launcher::new(minecraft_dir);

    let install = launcher.install(InstallRequest {
        minecraft_version: "1.20.4".to_string(),
        loader: Some(LoaderSpec::Fabric {
            version: LoaderVersion::LatestStable,
        }),
        java: JavaInstallPolicy::Auto,
    })?;

    let version_json_path = launcher
        .minecraft_dir()
        .join("versions")
        .join(&install.version_id)
        .join(format!("{}.json", install.version_id));
    let version_json: mc_launcher_core::core::version::VersionJson =
        serde_json::from_str(&std::fs::read_to_string(version_json_path)?)?;

    let command = launcher.build_launch_command_from_version(
        &version_json,
        LaunchOptions {
            account: Account::offline("Steve"),
            ..Default::default()
        },
    )?;

    let mut child = Command::new(command.executable)
        .args(command.args)
        .spawn()?;
    child.wait()?;
    Ok(())
}
