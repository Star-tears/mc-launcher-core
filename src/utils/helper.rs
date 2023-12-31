use chrono::Utc;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use lazy_static::lazy_static;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    sync::Mutex,
};

use crate::types::helper_types::RequestsResponseCache;

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

pub fn get_requests_response_cache(url: &str) -> Result<Value, reqwest::Error> {
    let mut cache = REQUESTS_RESPONSE_CACHE.lock().unwrap();
    if let Some(cache_entry) = cache.get(url) {
        let elapsed = Utc::now() - cache_entry.datetime;
        if elapsed.num_seconds() > 3600 {
            let response: Value = reqwest::blocking::get(url)?.json()?;
            let res = response.clone();
            let cache_entry = RequestsResponseCache {
                response,
                datetime: Utc::now(),
            };
            cache.insert(url.to_string(), cache_entry);
            return Ok(res);
        }
    }

    let response: Value = reqwest::blocking::get(url)?.json()?;
    let res = response.clone();
    let cache_entry = RequestsResponseCache {
        response,
        datetime: Utc::now(),
    };
    cache.insert(url.to_string(), cache_entry);
    Ok(res)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_agent() {
        let user_agent = get_user_agent();
        println!("Now user_agent: {:?}", user_agent);
    }

    #[test]
    fn test_get_requests_response_cache() {
        if let Ok(response) = get_requests_response_cache("https://httpbin.org/ip") {
            println!("{:#?}", response);
        }
    }
}
