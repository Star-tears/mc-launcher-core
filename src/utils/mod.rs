use std::env;
use std::path::PathBuf;

use crate::types::LatestMinecraftVersions;

use self::helper::get_requests_response_cache;

pub mod helper;
pub mod java;

pub fn get_minecraft_directory() -> PathBuf {
    let os = env::consts::OS;
    if os == "windows" {
        // Windows
        let home = env::var("APPDATA").unwrap_or_else(|_| {
            // If APPDATA is not set, use the default path
            let home = env::var("USERPROFILE").expect("USERPROFILE is not set");
            home
        });
        let appdata = format!(r"{}\.minecraft", home);
        PathBuf::from(appdata)
    } else if os == "macos" {
        // MacOS
        let home = env::var("HOME").expect("HOME is not set");
        PathBuf::from(format!("{}/Library/Application Support/minecraft", home))
    } else {
        // Other platforms (Linux and others)
        let home = env::var("HOME").expect("HOME is not set");
        PathBuf::from(format!("{}/.minecraft", home))
    }
}

pub fn get_latest_version() -> Result<LatestMinecraftVersions, Box<dyn std::error::Error>> {
    let response = get_requests_response_cache(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
    )?;
    let latest = response["latest"].clone();
    let release = latest["release"]
        .as_str()
        .ok_or_else(|| "Release version not found".to_string())?;
    let snapshot = latest["snapshot"]
        .as_str()
        .ok_or_else(|| "Snapshot version not found".to_string())?;

    Ok(LatestMinecraftVersions {
        release: release.to_string(),
        snapshot: snapshot.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_minecraft_directory() {
        let minecraft_directory = get_minecraft_directory();
        println!("Minecraft directory: {:?}", minecraft_directory);
    }

    #[test]
    fn test_get_latest_version() {
        if let Ok(latest_version) = get_latest_version() {
            println!("Minecraft latest_version: {:#?}", latest_version);
        }
    }
}
