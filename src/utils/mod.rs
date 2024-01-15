use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{env, fs};

use chrono::{DateTime, TimeZone, Utc};
use rand::Rng;
use serde_json::Value;
use uuid::Uuid;

use crate::types::shared_types::{ClientJson, VersionListManifestJson};
use crate::types::{
    Articles, LatestMinecraftVersions, MinecraftNewsOptions, MinecraftOptions, MinecraftVersionInfo,
};

use self::helper::get_requests_response_cache;

pub mod helper;
pub mod java;
pub mod natives;

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
    let res: VersionListManifestJson = serde_json::from_str(&response)?;
    let latest = res.latest;
    let release = latest.get("release").unwrap();
    let snapshot = latest.get("snapshot").unwrap();

    Ok(LatestMinecraftVersions {
        release: release.to_string(),
        snapshot: snapshot.to_string(),
    })
}

pub fn get_version_list() -> Result<Vec<MinecraftVersionInfo>, Box<dyn std::error::Error>> {
    let response = get_requests_response_cache(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
    )?;
    let vlist: VersionListManifestJson = serde_json::from_str(&response)?;
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
        Err(_) => return Ok(vec![]),
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
        let release_time = match DateTime::parse_from_rfc3339(
            &version_data.release_time.unwrap_or("0".to_string()),
        ) {
            Ok(time) => time.with_timezone(&Utc),
            Err(_) => Utc.timestamp_opt(0, 0).unwrap(),
        };
        let info = MinecraftVersionInfo {
            id: version_data.id.unwrap_or_default(),
            r#type: version_data.r#type.unwrap_or_default(),
            release_time,
            compliance_level: version_data.compliance_level.unwrap_or_default(),
        };

        version_list.push(info);
    }

    Ok(version_list)
}

pub fn get_available_versions<P: AsRef<Path>>(minecraft_directory: P) -> Vec<MinecraftVersionInfo> {
    let mut version_list = Vec::new();
    let mut version_check = HashSet::new();

    if let Ok(vlist) = get_version_list() {
        for i in vlist {
            version_list.push(i.clone());
            version_check.insert(i.id);
        }
    }
    if let Ok(installed_versions) = get_installed_versions(minecraft_directory) {
        for i in installed_versions {
            if !version_check.contains(&i.id) {
                version_list.push(i);
            }
        }
    }
    version_list
}

pub fn get_java_executable() -> Option<String> {
    let os = std::env::consts::OS;

    if os == "windows" {
        if let Some(java_home) = env::var_os("JAVA_HOME") {
            let java_home_path = Path::new(&java_home);
            let javaw_path = java_home_path.join("bin").join("javaw.exe");
            if javaw_path.is_file() {
                return Some(javaw_path.to_string_lossy().into_owned());
            }
        }

        if let Ok(_) =
            fs::metadata(r"C:\Program Files (x86)\Common Files\Oracle\Java\javapath\javaw.exe")
        {
            return Some(
                r"C:\Program Files (x86)\Common Files\Oracle\Java\javapath\javaw.exe".to_string(),
            );
        }

        if let Ok(javaw_path) = which::which("javaw") {
            return Some(javaw_path.to_string_lossy().into_owned());
        }
        return Some("javaw".to_string());
    } else if let Some(java_home) = env::var_os("JAVA_HOME") {
        let java_home_path = Path::new(&java_home);
        let java_path = java_home_path.join("bin").join("java");
        if java_path.is_file() {
            return Some(java_path.to_string_lossy().into_owned());
        }
    } else if os == "macos" || os == "ios" {
        if let Ok(java_path) = which::which("java") {
            return Some(java_path.to_string_lossy().into_owned());
        }
    } else {
        if let Ok(java_link) = fs::read_link("/etc/alternatives/java") {
            return Some(java_link.to_string_lossy().into_owned());
        } else if let Ok(runtime_link) = fs::read_link("/usr/lib/jvm/default-runtime") {
            let java_path = Path::new("/usr/lib/jvm")
                .join(runtime_link)
                .join("bin")
                .join("java");
            if java_path.is_file() {
                return Some(java_path.to_string_lossy().into_owned());
            }
        } else if let Ok(java_path) = which::which("java") {
            return Some(java_path.to_string_lossy().into_owned());
        }
    }

    Some("java".to_string())
}

// Return the version of mc-launcher-core
pub fn get_core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn generate_test_options() -> MinecraftOptions {
    let username = format!("Player{}", rand::thread_rng().gen_range(100..1000));
    let uuid = Uuid::new_v4();
    let token = String::new();

    MinecraftOptions::new(username, uuid.to_string(), token)
}

pub fn is_version_valid(version: &str, minecraft_directory: impl AsRef<Path>) -> bool {
    let versions_path = minecraft_directory.as_ref().join("versions");

    if versions_path.join(version).is_dir() {
        return true;
    }

    if let Ok(version_list) = get_version_list() {
        return version_list.iter().any(|i| i.id == version);
    }
    false
}

pub fn get_minecraft_news(options: MinecraftNewsOptions) -> Result<Articles, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://www.minecraft.net/content/minecraft-net/_jcr_content.articles.grid?pageSize={}",
        options.page_size
    );
    let user_agent = format!("mc-launcher-core/{}", get_core_version());

    let response: Articles = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()?
        .json()?;

    Ok(response)
}

pub fn is_vanilla_version(version: &str) -> bool {
    if let Ok(version_list) = get_version_list() {
        for i in version_list {
            if i.id == version {
                return true;
            }
        }
    }
    false
}

pub fn is_platform_supported() -> bool {
    match env::consts::OS {
        "windows" | "macos" | "linux" => true,
        _ => false,
    }
}

pub fn is_minecraft_installed(minecraft_directory: impl AsRef<Path>) -> bool {
    let versions_dir = minecraft_directory.as_ref().join("versions");
    let libraries_dir = minecraft_directory.as_ref().join("libraries");
    let assets_dir = minecraft_directory.as_ref().join("assets");

    versions_dir.is_dir() && libraries_dir.is_dir() && assets_dir.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_get_minecraft_directory() {
        let minecraft_directory = get_minecraft_directory();
        println!("Minecraft directory: {:?}", minecraft_directory);
    }

    #[test]
    fn debug_get_latest_version() {
        if let Ok(latest_version) = get_latest_version() {
            println!("Minecraft latest_version: {:#?}", latest_version);
        }
    }

    #[test]
    fn debug_get_version_list() {
        if let Ok(version_list) = get_version_list() {
            println!("Minecraft version_list: {:#?}", version_list);
        }
    }

    #[test]
    fn debug_get_installed_versions() {
        // match get_installed_versions(r"H:\mc\mc-launcher-core\test\.minecraft") {
        //     Ok(res) => {
        //         println!("Minecraft installed_versions: {:#?}", res);
        //     }
        //     Err(e) => println!("{:#?}", e),
        // }
    }

    #[test]
    fn debug_get_available_versions() {
        // println!(
        //     "Available versions: {:#?}",
        //     get_available_versions(r"H:\mc\mc-launcher-core\test\.minecraft")
        // );
    }

    #[test]
    fn debug_get_java_executable() {
        println!("Java excutable path: {:#?}", get_java_executable());
    }

    #[test]
    fn debug_get_core_version() {
        println!("mc-launcher-core version: {}", get_core_version());
    }

    #[test]
    fn debug_generate_test_options() {
        println!("Random MinecraftOptions: {:#?}", generate_test_options());
    }

    #[test]
    fn debug_is_version_valid() {
        // println!(
        //     "is_version_valid: {}",
        //     is_version_valid("1.20", r"H:\mc\mc-launcher-core\test\.minecraft")
        // );
    }

    #[test]
    fn debug_get_minecraft_news() {
        // let default_mcnews_options = MinecraftNewsOptions::default();
        // match get_minecraft_news(default_mcnews_options) {
        //     Ok(res) => println!("{:#?}", res),
        //     Err(e) => println!("{:#?}", e),
        // }
    }

    #[test]
    fn test_is_vanilla_version() {
        assert_eq!(is_vanilla_version("1.20"), true);
        assert_eq!(is_vanilla_version("20.24"), false);
    }

    #[test]
    fn debug_is_minecraft_installed() {
        // println!(
        //     "{}",
        //     is_minecraft_installed(r"H:\mc\mc-launcher-core\test\.minecraft")
        // );
    }
}
