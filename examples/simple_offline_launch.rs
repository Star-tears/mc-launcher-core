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
