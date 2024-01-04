use sysinfo::System;

use crate::types::shared_types::ClientJsonLibrary;

pub fn get_natives(data: &ClientJsonLibrary) -> String {
    let arch_type = if System::cpu_arch().unwrap_or_default() == "x86" {
        "32"
    } else {
        "64"
    };
    match &data.natives {
        Some(natives) => match std::env::consts::OS {
            "windows" => natives
                .get("windows")
                .map_or("".to_string(), |s| s.replace("${arch}", arch_type)),
            "macos" => natives
                .get("osx")
                .map_or("".to_string(), |s| s.replace("${arch}", arch_type)),
            _ => natives
                .get("linux")
                .map_or("".to_string(), |s| s.replace("${arch}", arch_type)),
        },
        None => "".to_string(),
    }
}
