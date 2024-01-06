use std::env;

use reqwest::header;
use serde_json::Value;

use crate::utils::helper::get_user_agent;

const JVM_MANIFEST_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

fn get_jvm_platform_string() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86") => "windows-x86".to_string(),
        ("windows", "aarch64") => "windows-arm64".to_string(),
        ("windows", _) => "windows-x64".to_string(),
        ("linux", "x86") => "linux-i386".to_string(),
        ("linux", _) => "linux".to_string(),
        ("macos", "aarch64") => "mac-os-arm64".to_string(),
        ("macos", _) => "mac-os".to_string(),
        _ => "gamecore".to_string(),
    }
}

pub fn get_jvm_runtimes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(JVM_MANIFEST_URL)
        .header(header::USER_AGENT, get_user_agent())
        .send()?;
    let manifest_data: Value = response.json()?;
    let platform_string = get_jvm_platform_string();
    if let Some(platform_data) = manifest_data.get(platform_string) {
        if let Some(platform_map) = platform_data.as_object() {
            let jvm_list: Vec<String> = platform_map.keys().cloned().collect();
            Ok(jvm_list)
        } else {
            Err("Invalid JSON format".into())
        }
    } else {
        Err("Platform not found in manifest".into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn debug_get_jvm_platform_string() {
        println!("{}", get_jvm_platform_string());
    }

    #[test]
    fn debug_get_jvm_runtimes() {
        match get_jvm_runtimes() {
            Ok(v) => println!("{:?}", v),
            Err(e) => println!("{}", e.to_string()),
        }
    }
}
