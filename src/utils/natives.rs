use std::{collections::HashMap, fs, io, path::Path};

use sysinfo::System;
use zip::ZipArchive;

use crate::types::{
    shared_types::{ClientJson, ClientJsonLibrary},
    MinecraftOptions,
};

use super::helper::{get_library_path, inherit_json, parse_rule_list};

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
    extract_path: impl AsRef<Path>,
    extract_data: &HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 如果提取目录不存在，则创建
    fs::create_dir_all(&extract_path)?;

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
            let mut output_file =
                fs::File::create(Path::new(extract_path.as_ref()).join(file.name()))?;
            io::copy(&mut file, &mut output_file)?;
        }
    }

    Ok(())
}

pub fn extract_natives(
    versionid: &str,
    path: impl AsRef<Path>,
    extract_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let version_file_path = path
        .as_ref()
        .join("versions")
        .join(versionid)
        .join(format!("{}.json", versionid));

    if !version_file_path.exists() {
        return Err(format!("version is not found: {}", versionid).into());
    }

    let json_content = fs::read_to_string(&version_file_path)?;
    let mut data: ClientJson = serde_json::from_str(&json_content)?;

    if data.inherits_from.is_some() {
        data = inherit_json(&data, &path)?;
    }

    for library in &data.libraries {
        if let Some(rules) = &library.rules {
            if !parse_rule_list(rules.to_vec(), MinecraftOptions::default()) {
                continue;
            }
        }

        let current_path = get_library_path(&library.name, &path);
        let native = get_natives(&library);

        if native.is_empty() {
            continue;
        }

        // 获取目录路径
        let lib_path = current_path.parent().unwrap_or_else(|| Path::new(""));
        let file_name = current_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        let lib_path_with_filename = lib_path.join(file_name);

        // 获取文件扩展名
        let extension = current_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let native_library_filename = format!(
            "{}-{}{}",
            lib_path_with_filename.to_string_lossy(),
            native,
            extension
        );

        extract_natives_file(
            &native_library_filename,
            &extract_path,
            library
                .extract
                .as_ref()
                .map_or(&HashMap::from([("exclude".to_string(), Vec::new())]), |e| e),
        )?;
    }

    Ok(())
}
