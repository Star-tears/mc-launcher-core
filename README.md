# mc-launcher-core

> This is an mc launcher core written in Rust.

You can use the functions provided by this library to create an mc launcher, offering only basic functionalities.

## Installation

Use `cargo add mc-launcher-core` to add it to your project.

## Example

Here's an example of installing Fabric and launching offline:

```rust
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

    let mut child = Command::new(command.executable).args(command.args).spawn()?;
    child.wait()?;
    Ok(())
}
```

## Todo list

- [x] Crate library
- [x] Install original version
- [x] Offline launch
- [x] Microsoft account login
- [x] Support for Forge, Fabric, Quilt, and NeoForge loader metadata
- [ ] Install of mrpack modpacks
- [ ] Comprehensive documentation

## Note

- There is still a lot of work to be done in this project. It is recommended to wait for version 0.1.0 before trying it out.
- The aim of this project is to have a user-friendly launcher SDK library written in Rust.
- During the development process, I referenced and learned from the minecraft-launcher-lib in Python libraries.
