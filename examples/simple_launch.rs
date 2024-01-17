use std::{io, process::Command};

use mc_launcher_core::{
    auth::microsoft_account::{complete_login, get_secure_login_data, parse_auth_code_url},
    command::get_minecraft_command,
    install::install_minecraft_version,
    types::{CallbackDict, MinecraftOptions},
    utils::{get_latest_version, get_minecraft_directory},
};

// Set the data for your Azure Application here.
const CLIENT_ID: &str = "YOUR CLIENT ID";
const REDIRECT_URI: &str = "YOUR REDIRECT URL";

fn main() {
    let latest_version = get_latest_version().unwrap().release;
    let minecraft_directory = get_minecraft_directory();

    let _ = install_minecraft_version(
        &latest_version,
        &minecraft_directory,
        &CallbackDict::default(),
    );

    let (login_url, state, code_verifier) = get_secure_login_data(CLIENT_ID, REDIRECT_URI, None);
    println!(
        "Please open {} in your browser and copy the url you are redirected into the prompt below.",
        login_url
    );
    let mut code_url = String::new();
    io::stdin().read_line(&mut code_url).expect("input error");

    let auth_code = parse_auth_code_url(&code_url, Some(state)).expect("Error get auth_code");

    let login_data = complete_login(
        CLIENT_ID,
        None,
        REDIRECT_URI,
        &auth_code,
        Some(&code_verifier),
    )
    .unwrap();

    let options = MinecraftOptions::new(login_data.name, login_data.id, login_data.access_token);

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
