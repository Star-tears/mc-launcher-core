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
```

`Launcher::install` performs a complete client install for vanilla, Fabric, and
Quilt profiles: version JSON, client jar, libraries, asset index, asset objects,
and native extraction are all handled by Rust library functions.

By default, launch commands use a version-isolated game directory at
`<minecraft_dir>/versions/<version_id>`. Pass `LaunchOptions::game_directory` to
use a custom instance directory.

## Compatibility

On macOS Apple Silicon, Minecraft versions that still use LWJGL 2 or older
LWJGL 3 metadata need extra compatibility handling. By default, launch command
building and download planning apply the needed macOS arm64 patches
automatically.

The patches replace old LWJGL, Java Objective-C bridge, and input-related
libraries with ManyMC/MinecraftMachina arm64-compatible metadata. The legacy
LWJGL 2 patch also adds missing JVM arguments such as `-XstartOnFirstThread`,
`-Djava.library.path`, and `-cp` for versions that still use
`minecraftArguments`.

The library does not bundle a Java runtime. For these legacy versions on Apple
Silicon, use an arm64 Java runtime matching the version metadata and pass it via
`LaunchOptions::java_executable`. Set `LaunchOptions::compatibility` or
`plan_vanilla_downloads_for_platform` to `CompatibilityPolicy::Disabled` if a
launcher wants to manage these patches itself.

The library separates library/native compatibility from window-hosting
compatibility. For legacy LWJGL 2 versions on macOS arm64,
`apply_compatibility` returns `CompatibilityResult::windowing` with
`WindowingStrategy::MacOsAppBundle` and
`requires_visible_window_verification = true`. This reflects the observed
behavior that a direct CLI-spawned Java process can initialize Minecraft and
still create a 0x0 invisible window. A desktop launcher should run these
versions from a real macOS app bundle or equivalent GUI host, then verify that a
visible game window was created.

## Todo list

- [x] Crate library
- [x] Install original version
- [x] Offline launch
- [x] Microsoft account login
- [x] Support for Forge, Fabric, Quilt, and NeoForge loader metadata
- [ ] Install of mrpack modpacks
- [ ] Comprehensive documentation

## Note

- Version 0.1.0 is the first release focused on complete vanilla/Fabric/Quilt client installation and version-isolated launches.
- The aim of this project is to have a user-friendly launcher SDK library written in Rust.
- During the development process, I referenced and learned from the minecraft-launcher-lib in Python libraries.
