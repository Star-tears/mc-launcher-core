use std::process::Command;

use mc_launcher_core::{
    auth::offline::get_offline_options,
    command::get_minecraft_command,
    utils::{get_latest_version, get_minecraft_directory},
};

fn main() {
    let options = get_offline_options("Steve");
    let minecraft_command_result = get_minecraft_command(
        &get_latest_version().unwrap().release,
        get_minecraft_directory(),
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
