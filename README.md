# mc-launcher-core

> This is an mc launcher core written in Rust.

You can use the functions provided by this library to create an mc launcher, offering only basic functionalities.

## Installation

Use `cargo add mc-launcher-core` to add it to your project.

## Example

Here's an example of launching the latest version of mc offline:

```rust
use std::process::Command;

use mc_launcher_core::{
    auth::offline::get_offline_options,
    command::get_minecraft_command,
    install::install_minecraft_version,
    types::CallbackDict,
    utils::{get_latest_version, get_minecraft_directory},
};

fn main() {
    let options = get_offline_options("Steve");
    let latest_version = get_latest_version().unwrap().release;
    let minecraft_directory = get_minecraft_directory();

    let _ = install_minecraft_version(
        &latest_version,
        &minecraft_directory,
        &CallbackDict::default(),
    );

    let minecraft_command_result = get_minecraft_command(
        &get_latest_version().unwrap().release,
        &minecraft_directory,
        &options,
    );

    match minecraft_command_result {
        Ok(minecraft_command) => {
            // Start Minecraft
            let mut cmd = Command::new(&minecraft_command[0])
                .args(&minecraft_command[1..minecraft_command.len()])
                .spawn()
                .expect("Failed to start Minecraft");
            let _ = cmd.wait();
        }
        Err(err) => {
            eprintln!("Failed to get Minecraft command: {}", err);
        }
    }
}

```

## Todo list

- [x] Crate library
- [x] Install original version
- [x] Offline launch
- [x] Microsoft account login
- [ ] Support for Forge, Fabric, Quilt, and Liteloader
- [ ] Install of mrpack modpacks
- [ ] Comprehensive documentation

## Note

- There is still a lot of work to be done in this project. It is recommended to wait for version 0.1.0 before trying it out.
- The aim of this project is to have a user-friendly launcher SDK library written in Rust.
- During the development process, I referenced and learned from the minecraft-launcher-lib in Python libraries.