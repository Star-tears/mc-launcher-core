use chrono::Utc;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::{Client, Response};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter, Read},
    path::{Path, PathBuf},
    sync::Mutex,
};
use sysinfo::System;
use winver::WindowsVersion;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::types::{
    exceptions_types::InvalidChecksum,
    helper_types::{MavenMetadata, RequestsResponseCache},
    shared_types::{ClientJson, ClientJsonRule, VersionListManifestJson},
    CallbackDict, MinecraftOptions,
};

pub fn check_path_inside_minecraft_directory(
    minecraft_directory: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let minecraft_directory = minecraft_directory.as_ref().canonicalize().unwrap();
    let path = path.as_ref().canonicalize().unwrap();

    if !path.starts_with(&minecraft_directory) {
        return Err(format!(
            "{} is outside Minecraft directory {}",
            path.to_string_lossy(),
            minecraft_directory.to_string_lossy()
        )
        .into());
    }
    Ok(())
}

pub fn download_file(
    url: &str,
    path: &str,
    sha1: Option<&str>,
    lzma_compressed: bool,
    minecraft_directory: Option<&str>,
    session: Option<reqwest::blocking::Client>,
    callback: CallbackDict,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(mc_dir) = minecraft_directory {
        check_path_inside_minecraft_directory(mc_dir, path)?;
    }

    if Path::new(path).is_file() {
        match sha1 {
            Some(expected_sha1) => {
                let actual_sha1 = get_sha1_hash(path)?;
                if actual_sha1 == expected_sha1 {
                    return Ok(false);
                }
            }
            None => return Ok(false),
        }
    }

    if let Some(parent_dir) = Path::new(path).parent() {
        if let Err(err) = fs::create_dir_all(parent_dir) {
            eprintln!("Error creating directories: {}", err);
        }
    }

    if let Some(set_status) = callback.set_status {
        if let Some(file_name) = Path::new(path).file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                set_status(format!("Download {}", file_name_str));
            }
        }
    }
    let mut response: Response;
    if session.is_none() {
        let client = Client::builder()
            .user_agent(get_user_agent()) // 设置 User-Agent
            .build()?;
        response = client.get(url).send()?;
    } else {
        response = Client::new().get(url).send()?;
    }

    if !response.status().is_success() {
        return Ok(false);
    }
    let mut file = BufWriter::new(File::create(path)?);
    if lzma_compressed {
        let mut decoder = XzDecoder::new(response);
        io::copy(&mut decoder, &mut file)?;
    } else {
        io::copy(&mut response, &mut file)?;
    }

    if let Some(expected_sha1) = sha1 {
        let actual_sha1 = get_sha1_hash(path)?;
        if actual_sha1 != expected_sha1 {
            return Err(Box::new(InvalidChecksum {
                url: url.to_string(),
                path: path.to_string(),
                expected: expected_sha1.to_string(),
                actual: actual_sha1,
            }));
        }
    }

    Ok(true)
}

fn parse_single_rule(rule: &ClientJsonRule, options: &MinecraftOptions) -> bool {
    let mut return_value = false;

    if rule.action == "allow" {
        return_value = false;
    } else if rule.action == "disallow" {
        return_value = true;
    }

    if let Some(os) = &rule.os {
        if let Some(name) = os.get("name") {
            if name == "windows" && std::env::consts::OS != "windows" {
                return return_value;
            } else if name == "osx" && std::env::consts::OS != "macos" {
                return return_value;
            } else if name == "linux" && std::env::consts::OS != "linux" {
                return return_value;
            }
        }
        if let Some(arch) = os.get("arch") {
            if let Some(arch_info) = System::cpu_arch() {
                if arch == "x86" && arch_info != "x86" {
                    return return_value;
                }
            }
        }
        if let Some(version) = os.get("version") {
            let os_version = get_os_version();
            let re = Regex::new(version).unwrap();
            if !re.is_match(&os_version) {
                return return_value;
            }
        }
    }

    if let Some(features) = &rule.features {
        if let Some(_) = features.get("has_custom_resolution") {
            if !options.custom_resolution.unwrap_or(false) {
                return return_value;
            }
        }
        if let Some(_) = features.get("is_demo_user") {
            if !options.demo.unwrap_or(false) {
                return return_value;
            }
        }
        if let Some(_) = features.get("has_quick_plays_support") {
            if options.quick_play_path.is_none() {
                return return_value;
            }
        }
        if let Some(_) = features.get("is_quick_play_singleplayer") {
            if options.quick_play_singleplayer.is_none() {
                return return_value;
            }
        }
        if let Some(_) = features.get("is_quick_play_multiplayer") {
            if options.quick_play_multiplayer.is_none() {
                return return_value;
            }
        }
        if let Some(_) = features.get("is_quick_play_realms") {
            if options.quick_play_realms.is_none() {
                return return_value;
            }
        }
    }

    !return_value
}

pub fn parse_rule_list(rules: Vec<ClientJsonRule>, options: MinecraftOptions) -> bool {
    for i in rules {
        if !parse_single_rule(&i, &options) {
            return false;
        }
    }
    true
}

pub fn inherit_json(
    original_data: &ClientJson,
    path: impl AsRef<Path>,
) -> Result<ClientJson, Box<dyn std::error::Error>> {
    let inherit_version = original_data
        .inherits_from
        .as_ref()
        .ok_or("Missing 'inheritsFrom' key")?;

    let mut file_path = path.as_ref().canonicalize()?;
    file_path.push("versions");
    file_path.push(inherit_version);
    file_path.push(format!("{}.json", inherit_version));

    let mut file = File::open(file_path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    let mut new_data: ClientJson = serde_json::from_str(&file_content)?;

    new_data.merge(original_data);

    Ok(new_data)
}

pub fn get_library_path(name: &str, path: impl AsRef<Path>) -> PathBuf {
    let mut libpath = path.as_ref().join("libraries");
    let parts: Vec<&str> = name.split(":").collect();
    let (base_path, libname, version) = match &parts[..3] {
        [base_path, libname, version] => (base_path, libname, version),
        _ => panic!("无效的库名称格式"),
    };
    for i in base_path.split('.') {
        libpath = libpath.join(i);
    }
    let (version, fileend) = match version.split_once('@') {
        Some((version, fileend)) => (version, fileend),
        None => ("", "jar"),
    };
    let filename = format!(
        "{}-{}{}.{}",
        libname,
        version,
        parts
            .iter()
            .skip(3)
            .map(|p| format!("-{}", p))
            .collect::<String>(),
        fileend
    );
    libpath.join(libname).join(version).join(filename)
}

pub fn get_jar_mainclass(path: impl AsRef<Path>) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut manifest = String::new();

    // Read the MANIFEST.MF file from the JAR
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().eq_ignore_ascii_case("META-INF/MANIFEST.MF") {
            entry.read_to_string(&mut manifest)?;
            break;
        }
    }

    // Parse the MANIFEST.MF content to find the Main-Class
    let main_class = manifest
        .lines()
        .find(|line| line.starts_with("Main-Class:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim())
        .ok_or("Main-Class not found in MANIFEST.MF")?;

    Ok(main_class.to_string())
}

pub fn get_sha1_hash(path: impl AsRef<Path>) -> Result<String, Box<dyn std::error::Error>> {
    const BUF_SIZE: usize = 65536;
    let mut file = File::open(path)?;
    let mut buffer = [0; BUF_SIZE];
    let mut sha1 = Sha1::new();

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        sha1.input(&buffer[..bytes_read]);
    }

    Ok(sha1.result_str())
}

pub fn get_os_version() -> String {
    if std::env::consts::OS == "windows" {
        let version = WindowsVersion::detect().unwrap();
        return format!("{}.{}", version.major, version.minor);
    }
    System::os_version().expect("failed get os version")
}

lazy_static! {
    static ref REQUESTS_RESPONSE_CACHE: Mutex<HashMap<String, RequestsResponseCache>> =
        Mutex::new(HashMap::new());
    static ref USER_AGENT_CACHE: Mutex<Option<String>> = Mutex::new(None);
}

// return the user agent of mc-launcher-core
pub fn get_user_agent() -> String {
    let mut cache = USER_AGENT_CACHE.lock().unwrap();
    if let Some(ref user_agent) = *cache {
        return user_agent.clone();
    } else {
        let user_agent = "mc-launcher-core/skycrafting".to_string();
        *cache = Some(user_agent.clone());
        return user_agent;
    }
}

pub fn get_requests_response_cache(url: &str) -> Result<String, reqwest::Error> {
    let mut cache = REQUESTS_RESPONSE_CACHE.lock().unwrap();
    if let Some(cache_entry) = cache.get(url) {
        let elapsed = Utc::now() - cache_entry.datetime;
        if elapsed.num_seconds() > 3600 {
            let response = reqwest::blocking::get(url)?.text()?;
            let res = response.clone();
            let cache_entry = RequestsResponseCache {
                response,
                datetime: Utc::now(),
            };
            cache.insert(url.to_string(), cache_entry);
            return Ok(res);
        }
    }

    let response = reqwest::blocking::get(url)?.text()?;
    let res = response.clone();
    let cache_entry = RequestsResponseCache {
        response,
        datetime: Utc::now(),
    };
    cache.insert(url.to_string(), cache_entry);
    Ok(res)
}

pub fn get_classpath_separator() -> String {
    if std::env::consts::OS == "windows" {
        ";".to_string()
    } else {
        ":".to_string()
    }
}

pub fn parse_maven_metadata(url: &str) -> Result<MavenMetadata, Box<dyn std::error::Error>> {
    let response = get_requests_response_cache(url)?;

    let release_regex = Regex::new(r#"<release>(.*?)</release>"#)?;
    let latest_regex = Regex::new(r#"<latest>(.*?)</latest>"#)?;
    let version_regex = Regex::new(r#"<version>(.*?)</version>"#)?;

    let release = release_regex
        .captures(&response)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();
    let latest = latest_regex
        .captures(&response)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();
    let versions: Vec<String> = version_regex
        .captures_iter(&response)
        .map(|m| m.get(1).unwrap().as_str().to_string())
        .collect();

    Ok(MavenMetadata {
        release,
        latest,
        versions,
    })
}

pub fn extract_file_from_zip(
    handler: &mut zip::ZipArchive<std::fs::File>,
    zip_path: &str,
    extract_path: &str,
    minecraft_directory: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(minecraft_directory) = minecraft_directory {
        check_path_inside_minecraft_directory(minecraft_directory, extract_path)?;
    }

    if let Some(parent) = Path::new(extract_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(extract_path)?;
    let mut zip_file = handler.by_name(zip_path)?;
    io::copy(&mut zip_file, &mut file)?;

    Ok(())
}

pub fn get_client_json(
    version: &str,
    minecraft_directory: impl AsRef<Path>,
) -> Result<ClientJson, Box<dyn std::error::Error>> {
    let local_path = minecraft_directory
        .as_ref()
        .join("versions")
        .join(version)
        .join(format!("{}.json", version));
    if local_path.exists() {
        let file = fs::File::open(&local_path)?;
        let data: ClientJson = serde_json::from_reader(file)?;

        if data.inherits_from.is_some() {
            let inherited_data = inherit_json(&data, minecraft_directory.as_ref())?;
            return Ok(inherited_data);
        }

        return Ok(data);
    }

    let version_manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
    let version_manifest: VersionListManifestJson =
        serde_json::from_str(get_requests_response_cache(version_manifest_url)?.as_str())?;
    if let Some(version_info) = version_manifest.versions.iter().find(|&v| v.id == version) {
        let version_url = &version_info.url;
        let version_data: ClientJson =
            serde_json::from_str(get_requests_response_cache(version_url)?.as_str())?;
        return Ok(version_data);
    }

    Err(format!("version is not found: {}", version).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_get_user_agent() {
        let user_agent = get_user_agent();
        println!("Now user_agent: {:?}", user_agent);
    }

    #[test]
    fn debug_get_requests_response_cache() {
        if let Ok(response) = get_requests_response_cache("https://httpbin.org/ip") {
            println!("{:#?}", response);
        }
    }

    #[test]
    fn debug_parse_maven_metadata() {
        // let url =
        //     "https://files.minecraftforge.net/maven/net/minecraftforge/forge/maven-metadata.xml";
        // match parse_maven_metadata(url) {
        //     Ok(res) => println!("{:?}", res),
        //     Err(e) => println!("{:#?}", e),
        // }
    }

    #[test]
    fn debug_get_os_version() {
        println!("{}", get_os_version());
    }

    #[test]
    fn debug_get_jar_mainclass() {
        // match get_jar_mainclass(
        //     r"H:\mc\mc-launcher-core\test\.minecraft\versions\DarkRPG FORGE - RPG, Quest, Magic, Dark Souls\DarkRPG FORGE - RPG, Quest, Magic, Dark Souls.jar",
        // ) {
        //     Ok(s) => println!("jar mainclass: {}", s),
        //     Err(e) => println!("{}", e.to_string()),
        // }
    }

    #[test]
    fn debug_get_client_json() {
        match get_client_json("1.19", r"H:\mc\mc-launcher-core\test\.minecraft") {
            Ok(client_json) => println!("{:#?}", client_json),
            Err(e) => println!("{}", e.to_string()),
        }
    }
}
