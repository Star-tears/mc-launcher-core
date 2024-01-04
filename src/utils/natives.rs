use std::{collections::HashMap, fs, io, path::Path};

use sysinfo::System;
use zip::ZipArchive;

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

pub fn extract_natives_file(
    filename: &str,
    extract_path: &str,
    extract_data: &HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 如果提取目录不存在，则创建
    fs::create_dir_all(extract_path)?;

    // 打开 ZIP 文件
    let file = fs::File::open(filename)?;
    let mut archive = ZipArchive::new(file)?;

    // 遍历 ZIP 存档中的每个文件
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // 检查是否应排除文件
        let mut should_exclude = false;
        for e in extract_data.get("exclude").expect("not exist key: exclude") {
            if file.name().starts_with(e) {
                should_exclude = true;
                break;
            }
        }

        // 如果不应排除文件，则提取文件
        if !should_exclude {
            let mut output_file = fs::File::create(Path::new(extract_path).join(file.name()))?;
            io::copy(&mut file, &mut output_file)?;
        }
    }

    Ok(())
}
