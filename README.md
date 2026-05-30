# mc-launcher-core

`mc-launcher-core` is a Rust library for building Minecraft launchers. It
handles the install and launch-preparation work that sits behind a launcher UI:
resolving version metadata, installing client files, preparing loader profiles,
extracting natives, and building a structured Java launch command.

The crate is designed as a launcher SDK. It does not provide a GUI and it does
not spawn Minecraft automatically unless your application chooses to use the
returned command with `std::process::Command`.

## Features

- Install vanilla Minecraft profiles.
- Install Fabric, Quilt, Forge, and NeoForge profiles.
- Download client jars, libraries, asset indexes, asset objects, and natives.
- Merge inherited version metadata before launch.
- Build cross-platform Java launch commands without shell string quoting.
- Use offline accounts or Microsoft account helper APIs.
- Report install progress through a simple callback trait.
- Apply macOS Apple Silicon compatibility patches for older LWJGL metadata.
- Keep compatibility wrappers for older `mc-launcher-core` callers.

## Installation

```bash
cargo add mc-launcher-core
```

Or add it manually:

```toml
[dependencies]
mc-launcher-core = "0.1.1"
```

## Quick Start

Install Fabric and launch with an offline account:

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

    let version = launcher.load_version(&install.version_id)?;
    let command = launcher.build_launch_command_from_version(
        &version,
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

## Main API

Use `mc_launcher_core::prelude::*` for the common facade:

- `Launcher` owns a Minecraft directory and coordinates install/load/launch work.
- `InstallRequest` describes the Minecraft version and optional loader.
- `LoaderSpec` and `LoaderVersion` select Fabric, Quilt, Forge, or NeoForge.
- `LaunchOptions` controls account, Java path, game directory, resolution, server,
  and compatibility behavior.
- `LaunchCommand` contains `executable`, `args`, `working_dir`, and `env`.
- `ProgressReporter` receives install/download progress events.
- `LauncherError` and `Result` are the crate-wide error types.

## Installing Versions

Vanilla:

```rust
use mc_launcher_core::prelude::*;

fn main() -> mc_launcher_core::Result<()> {
    let launcher = Launcher::new(".minecraft");
    let install = launcher.install(InstallRequest::vanilla("1.20.1"))?;
    println!("installed profile: {}", install.version_id);
    Ok(())
}
```

Fabric, Quilt, Forge, and NeoForge use the same request shape:

```rust
use mc_launcher_core::prelude::*;

fn main() -> mc_launcher_core::Result<()> {
    let launcher = Launcher::new(".minecraft");
    let install = launcher.install(InstallRequest {
        minecraft_version: "1.20.1".to_string(),
        loader: Some(LoaderSpec::Quilt {
            version: LoaderVersion::Latest,
        }),
        java: JavaInstallPolicy::Auto,
    })?;
    println!("installed profile: {}", install.version_id);
    Ok(())
}
```

Use `LoaderVersion::Exact("...".to_string())` when a launcher UI lets users pick
a specific loader version.

## Progress Reporting

`Launcher::install_with_progress` accepts any closure that takes a
`ProgressEvent`:

```rust
use mc_launcher_core::prelude::*;

fn main() -> mc_launcher_core::Result<()> {
    let launcher = Launcher::new(".minecraft");
    let mut progress = |event: ProgressEvent| {
        println!("{event:?}");
    };

    launcher.install_with_progress(InstallRequest::vanilla("1.20.1"), &mut progress)?;
    Ok(())
}
```

## Launch Options

`LaunchOptions::default()` is suitable for a basic offline launch. Override only
what your launcher exposes:

```rust
use std::path::PathBuf;

use mc_launcher_core::prelude::*;

let options = LaunchOptions {
    account: Account::offline("Steve"),
    java_executable: Some(PathBuf::from("/path/to/java")),
    game_directory: Some(PathBuf::from(".minecraft/instances/survival")),
    custom_resolution: Some((1280, 720)),
    server: Some(("example.org".to_string(), Some(25565))),
    ..Default::default()
};
```

By default, the game directory is isolated per installed profile at
`<minecraft_dir>/versions/<version_id>`. Set `game_directory` if your launcher
has separate instance folders.

## Java Runtime

The new facade does not bundle a Java runtime. Select a runtime in your
application and pass it through `LaunchOptions::java_executable`. If omitted,
the generated command uses `java` from `PATH`.

Forge and NeoForge installer execution currently invokes `java` while
installing. Future runtime management can use the existing `JavaInstallPolicy`
request field.

## Compatibility

On macOS Apple Silicon, older Minecraft metadata may reference LWJGL libraries
that do not work on arm64. By default, command construction and download
planning use `CompatibilityPolicy::Auto` and apply known patches.

The compatibility layer can:

- replace legacy LWJGL 2 metadata with arm64-capable libraries;
- replace older LWJGL 3 metadata with macOS arm64 native classifiers;
- recommend a matching arm64 Java runtime;
- report when legacy LWJGL 2 should be hosted from a real macOS app bundle.

Disable automatic patching only if your launcher manages this itself:

```rust
use mc_launcher_core::prelude::*;

let options = LaunchOptions {
    compatibility: CompatibilityPolicy::Disabled,
    ..Default::default()
};
```

## Microsoft Login

Offline launch is available through `Account::offline`. Microsoft login helpers
live in `mc_launcher_core::auth::microsoft_account` and expose the individual
OAuth, Xbox Live, XSTS, and Minecraft services steps.

For new apps, start with `get_secure_login_data` so the login URL uses PKCE.
After the browser redirects back to your app, parse the auth code and call
`complete_login`.

## Lower-Level Modules

The facade is intentionally small, but the crate also exposes lower-level
modules for custom launchers:

- `core::version` contains the normalized version JSON model.
- `install::vanilla`, `install::libraries`, and `install::assets` build download
  plans without immediately running them.
- `net::download` executes structured download plans with checksum validation.
- `io::paths` and `io::archive` provide path-safe filesystem helpers.
- `loader::{fabric, quilt, forge, neoforge}` expose loader metadata helpers.

Generate local rustdoc for the full API:

```bash
cargo doc --no-deps --open
```

## Examples

The repository includes runnable examples:

```bash
cargo run --example simple_offline_launch
cargo run --example simple_launch
cargo run --example launch_matrix
```

The examples download real Minecraft metadata and files, so they require
network access and enough disk space for the selected versions.

## Status

- Crate library: done
- Vanilla install: done
- Offline launch: done
- Microsoft account login helpers: done
- Forge, Fabric, Quilt, and NeoForge loader metadata: done
- mrpack modpack install: planned
- Rustdoc and README documentation: done

## Notes

- Version `0.1.1` focuses on complete vanilla/Fabric/Quilt client installation,
  Forge/NeoForge installer support, and version-isolated launches.
- This project aims to provide a user-friendly launcher SDK written in Rust.
- The project was inspired in part by the Python `minecraft-launcher-lib`
  ecosystem.
