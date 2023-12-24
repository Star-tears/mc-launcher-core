use chrono::Utc;
use lazy_static::lazy_static;
use serde_json::Value;
use std::{collections::HashMap, env, sync::Mutex};

use crate::types::helper_types::RequestsResponseCache;

// return the user agent of mc-launcher-core
pub fn get_user_agent() -> String {
    let user_agent_cache = std::env::var("USER_AGENT_CACHE").unwrap_or_else(|_| {
        env::set_var("MC_LAUNCHER_CORE", "skycrafting");
        "mc-launcher-core/skycrafting".to_string()
    });
    user_agent_cache
}

lazy_static! {
    static ref REQUESTS_RESPONSE_CACHE: Mutex<HashMap<String, RequestsResponseCache>> =
        Mutex::new(HashMap::new());
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
        if let Ok(response) = get_requests_response_cache(
            "https://httpbin.org/ip",
        ) {
            println!("{:?}", response);
        }
    }
}
