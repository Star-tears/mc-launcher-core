use std::path::{Path, PathBuf};
use std::{env, fs, io};

use chrono::{DateTime, TimeZone, Utc};

use crate::types::shared_types::{ClientJson, VersionListManifestJson};
use crate::types::{LatestMinecraftVersions, MinecraftVersionInfo};

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

pub fn get_version_list() -> Result<Vec<MinecraftVersionInfo>, Box<dyn std::error::Error>> {
    let response = get_requests_response_cache(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
    )?;
    let vlist: VersionListManifestJson = serde_json::from_value(response)?;
    let mut res = Vec::new();
    for v in vlist.versions {
        res.push(MinecraftVersionInfo {
            id: v.id,
            r#type: v.r#type,
            release_time: DateTime::parse_from_rfc3339(v.release_time.as_str())?.into(),
            compliance_level: v.compliance_level,
        })
    }
    Ok(res)
}

pub fn get_installed_versions(
    minecraft_directory: impl AsRef<Path>,
) -> Result<Vec<MinecraftVersionInfo>, Box<dyn std::error::Error>> {
    let versions_path = minecraft_directory.as_ref().join("versions");
    let dir_list = match fs::read_dir(versions_path) {
        Ok(dir) => dir,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(Box::new(e)),
    };
    let mut version_list = Vec::new();
    for entry in dir_list {
        let entry = entry?;
        let mut dir_name = entry.file_name().to_string_lossy().to_string();
        dir_name.push_str(".json");
        let path = entry.path().join(dir_name);
        if !path.is_file() || path.extension().unwrap_or_default() != "json" {
            continue;
        }
        let file_content = fs::read_to_string(&path)?;
        let version_data: ClientJson = serde_json::from_str(file_content.as_str())?;
        let release_time = match DateTime::parse_from_rfc3339(&version_data.release_time) {
            Ok(time) => time.with_timezone(&Utc),
            Err(_) => Utc.timestamp_opt(0, 0).unwrap(),
        };
        let info = MinecraftVersionInfo {
            id: version_data.id,
            r#type: version_data.r#type,
            release_time,
            compliance_level: version_data.compliance_level,
        };

        version_list.push(info);
    }

    Ok(version_list)
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

    #[test]
    fn test_get_version_list() {
        if let Ok(version_list) = get_version_list() {
            println!("Minecraft version_list: {:#?}", version_list);
        }
    }

    #[test]
    fn test_get_installed_versions() {
        // match get_installed_versions(r"H:\mc\PCL\Release 2.3.0\.minecraft") {
        //     Ok(res) => {
        //         println!("Minecraft installed_versions: {:#?}", res);
        //     }
        //     Err(e) => println!("{:#?}", e),
        // }
    }
}
