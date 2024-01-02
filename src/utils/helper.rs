use chrono::Utc;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap, env, fs::File, io::Read, path::Path, process::Command, sync::Mutex,
};
use sysinfo::System;

use crate::types::helper_types::{MavenMetadata, RequestsResponseCache};

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
