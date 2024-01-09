use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use chrono::DateTime;
use reqwest::header;

use crate::{
    types::{
        runtime_types::{PlatformManifestJson, RuntimeListJson},
        CallbackDict, JvmRuntimeInformation,
    },
    utils::helper::{
        check_path_inside_minecraft_directory, download_file, get_sha1_hash, get_user_agent,
    },
};

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
    let manifest_data: RuntimeListJson = response.json()?;
    let platform_string = get_jvm_platform_string();
    if let Some(platform_data) = manifest_data.get(&platform_string) {
        let jvm_list: Vec<String> = platform_data.keys().cloned().collect();
        Ok(jvm_list)
    } else {
        Err("Platform not found in manifest".into())
    }
}

pub fn get_installed_jvm_runtimes(minecraft_directory: impl AsRef<Path>) -> Vec<String> {
    let runtime_dir = minecraft_directory.as_ref().join("runtime");
    match fs::read_dir(runtime_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn install_jvm_runtime(
    jvm_version: &str,
    minecraft_directory: impl AsRef<Path>,
    callback: &CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let manifest_data: RuntimeListJson = client
        .get(JVM_MANIFEST_URL)
        .header(header::USER_AGENT, get_user_agent())
        .send()?
        .json()?;
    let platform_string = get_jvm_platform_string();

    // Check if the JVM version exists
    if !manifest_data
        .get(&platform_string)
        .unwrap_or(&HashMap::new())
        .contains_key(jvm_version)
    {
        return Err(format!("jvm version not found: {}", jvm_version).into());
    }

    // Check if there is a platform manifest
    if manifest_data
        .get(&platform_string)
        .unwrap_or(&HashMap::new())
        .get(jvm_version)
        .unwrap_or(&Vec::new())
        .len()
        == 0
    {
        return Err("platform manifest not exist.".into());
    }
    let platform_manifest_url = manifest_data
        .get(&platform_string)
        .unwrap()
        .get(jvm_version)
        .unwrap()[0]
        .manifest
        .url
        .clone();
    let platform_manifest: PlatformManifestJson = client
        .get(platform_manifest_url)
        .header(header::USER_AGENT, get_user_agent())
        .send()?
        .json()?;
    let base_path = minecraft_directory
        .as_ref()
        .join("runtime")
        .join(jvm_version)
        .join(&platform_string)
        .join(jvm_version);

    // Download all files of the runtime
    if let Some(set_max) = callback.set_max {
        set_max(platform_manifest.files.len() as i32 - 1);
    }
    let mut count = 0;
    let mut file_list: Vec<&String> = vec![];
    for (key, value) in platform_manifest.files.iter() {
        let current_path = base_path.join(key);
        check_path_inside_minecraft_directory(&minecraft_directory, &current_path);
        if let Some(vtype) = &value.r#type {
            if vtype == "file" {
                if let Some(download_info) = &value.downloads {
                    if download_info.contains_key("lzma") {
                        download_file(
                            &download_info.get("lzma").unwrap().url,
                            &current_path,
                            Some(download_info.get("raw").unwrap().sha1.as_str()),
                            true,
                            None,
                            Some(&client),
                            callback,
                        )?;
                    } else {
                        download_file(
                            &download_info.get("raw").unwrap().url,
                            &current_path,
                            Some(download_info.get("raw").unwrap().sha1.as_str()),
                            false,
                            None,
                            Some(&client),
                            callback,
                        )?;
                    }
                }
                //Make files executable on unix systems
                if value.executable == Some(true) {
                    let _ = Command::new("chmod").arg("+x").arg(current_path).status();
                }
                file_list.push(key);
            } else if vtype == "directory" {
                let _ = fs::create_dir_all(&current_path);
            } else if vtype == "link" {
                check_path_inside_minecraft_directory(
                    &minecraft_directory,
                    base_path.join(&value.target.as_ref().map_or("".to_string(), |s| s.clone())),
                );
                if !current_path.parent().unwrap().exists() {
                    let _ = fs::create_dir_all(current_path.parent().unwrap());
                }
                // Create a symbolic link at `link_path` pointing to `target`
                #[cfg(unix)]
                {
                    let _ =
                        std::os::unix::fs::symlink(Path::new(value.target.unwrap()), &current_path);
                }
            }
            if let Some(set_progresss) = callback.set_progress {
                set_progresss(count);
            }
            count += 1;
        }
    }
    // Create the .version file
    let version_path = minecraft_directory
        .as_ref()
        .join("runtime")
        .join(jvm_version)
        .join(&platform_string)
        .join(".version");
    check_path_inside_minecraft_directory(&minecraft_directory, &version_path);
    let mut version_file = fs::File::create(&version_path)?;
    version_file.write_all(
        manifest_data
            .get(&platform_string)
            .unwrap()
            .get(jvm_version)
            .unwrap()[0]
            .version
            .get("name")
            .unwrap()
            .as_bytes(),
    )?;

    // Write the .sha1 file
    let sha1_path = minecraft_directory
        .as_ref()
        .join("runtime")
        .join(jvm_version)
        .join(platform_string)
        .join(format!("{}.sha1", jvm_version));
    check_path_inside_minecraft_directory(&minecraft_directory, &sha1_path);
    let mut sha1_file = fs::File::create(&sha1_path)?;
    for file in file_list {
        let current_path = base_path.join(file);
        let ctime = current_path.metadata()?.modified()?.elapsed()?.as_nanos(); // Use chrono for more precise time handling
        let sha1 = get_sha1_hash(current_path.to_str().unwrap())?;
        sha1_file.write_all(format!("{} /#// {} {}\n", file, sha1, ctime).as_bytes())?;
    }
    Ok(())
}

pub fn get_executable_path(
    jvm_version: &str,
    minecraft_directory: impl AsRef<Path>,
) -> Option<PathBuf> {
    let java_path = minecraft_directory
        .as_ref()
        .join("runtime")
        .join(jvm_version)
        .join(get_jvm_platform_string())
        .join(jvm_version)
        .join("bin")
        .join("java");

    if java_path.is_file() {
        return Some(java_path);
    }

    let java_exe_path = java_path.with_extension("exe");
    if java_exe_path.is_file() {
        return Some(java_exe_path);
    }

    let java_alternate_path = minecraft_directory
        .as_ref()
        .join("runtime")
        .join(jvm_version)
        .join(get_jvm_platform_string())
        .join(jvm_version)
        .join("jre.bundle")
        .join("Contents")
        .join("Home")
        .join("bin")
        .join("java");

    if java_alternate_path.is_file() {
        return Some(java_alternate_path);
    }

    None
}

pub fn get_jvm_runtime_information(
    jvm_version: &str,
) -> Result<JvmRuntimeInformation, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let manifest_data: RuntimeListJson = client
        .get(JVM_MANIFEST_URL)
        .header("user-agent", get_user_agent())
        .send()?
        .json()?;

    let platform_string = get_jvm_platform_string();

    // Check if the jvm version exists
    if !manifest_data
        .get(&platform_string)
        .unwrap_or(&HashMap::new())
        .contains_key(jvm_version)
    {
        return Err(format!("jvm version is not found: {}", jvm_version).into());
    }

    if manifest_data
        .get(&platform_string)
        .unwrap()
        .get(jvm_version)
        .unwrap_or(&Vec::new())
        .is_empty()
    {
        return Err(format!("this platform not supported yet.").into());
    }
    let runtime_list_json_entry = manifest_data
        .get(&platform_string)
        .unwrap()
        .get(jvm_version)
        .unwrap();
    Ok(JvmRuntimeInformation {
        name: runtime_list_json_entry[0]
            .version
            .get("name")
            .unwrap()
            .to_string(),
        released: DateTime::parse_from_rfc3339(
            runtime_list_json_entry[0].version.get("released").unwrap(),
        )?
        .into(),
    })
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
