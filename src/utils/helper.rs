use chrono::Utc;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::{Client, Response};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::Path,
    process::Command,
    sync::Mutex,
};
use sysinfo::System;
use xz2::read::XzDecoder;

use crate::types::{
    exceptions_types::InvalidChecksum,
    helper_types::{MavenMetadata, RequestsResponseCache},
    CallbackDict,
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
        let url =
            "https://files.minecraftforge.net/maven/net/minecraftforge/forge/maven-metadata.xml";
        // match parse_maven_metadata(url) {
        //     Ok(res) => println!("{:?}", res),
        //     Err(e) => println!("{:#?}", e),
        // }
    }

    #[test]
    fn debug_get_os_version() {
        println!("{}", get_os_version());
    }
}
