use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::types::JavaInformation;

pub fn get_java_information(path: &Path) -> Result<JavaInformation, String> {
    let is_windows = env::consts::OS == "windows";
    let java_exe = if is_windows { "java.exe" } else { "java" };

    let java_path = path.join("bin").join(java_exe);
    if !java_path.exists() {
        return Err(format!("{} was not found", java_path.display()));
    }

    let output = Command::new(&java_path)
        .arg("-showversion")
        .output()
        .map_err(|e| format!("Failed to execute Java: {}", e))?;

    let output_str = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = output_str.lines().collect();

    let version = lines[0]
        .split("version \"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .ok_or_else(|| "Failed to parse Java version".to_string())?;
    let javaw_path = if is_windows {
        let tmp_javaw_path = path
            .join("bin")
            .join("javaw.exe")
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or_default();
        Some(tmp_javaw_path)
    } else {
        None
    };
    Ok(JavaInformation::new(
        path.to_str().unwrap_or_default(),
        path.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
        version,
        lines[2].contains("64-Bit"),
        lines[0].starts_with("openjdk"),
        java_path.to_str().unwrap_or_default(),
        javaw_path,
    ))
}

fn search_java_directory<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    // 检查路径是否是一个目录
    if let Ok(entries) = fs::read_dir(&path) {
        // 使用迭代器和 filter 进行条件筛选
        let java_list: Vec<PathBuf> = entries
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    let current_entry = entry.path();

                    // 如果是目录，并且包含 "bin/java" 或 "bin/java.exe" 文件，将其添加到 Java 列表中
                    if current_entry.is_dir()
                        && (current_entry.join("bin/java").is_file()
                            || current_entry.join("bin/java.exe").is_file())
                    {
                        Some(current_entry)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        return java_list;
    }

    // 如果不是目录，或者读取目录失败，返回一个空列表
    Vec::new()
}

pub fn find_system_java_versions<P: AsRef<Path>>(
    additional_directories: Option<Vec<P>>,
) -> Vec<PathBuf> {
    let os = env::consts::OS;
    let mut java_list: Vec<PathBuf> = Vec::new();

    if os == "windows" {
        java_list.extend_from_slice(&search_java_directory(r"C:\Program Files (x86)\Java"));
        java_list.extend_from_slice(&search_java_directory(r"C:\Program Files\Java"));
    } else if os == "linux" {
        java_list.extend_from_slice(&search_java_directory("/usr/lib/jvm"));
        java_list.extend_from_slice(&search_java_directory("/usr/lib/sdk"));
    }

    if let Some(additional_directories) = additional_directories {
        for i in additional_directories {
            java_list.extend_from_slice(&search_java_directory(i.as_ref()));
        }
    }

    java_list
}

pub fn find_system_java_versions_information<P: AsRef<Path>>(
    additional_directories: Option<Vec<P>>,
) -> Vec<JavaInformation> {
    let mut java_information_list: Vec<JavaInformation> = Vec::new();

    for i in find_system_java_versions(additional_directories) {
        if let Ok(java_information) = get_java_information(&i) {
            java_information_list.push(java_information);
        }
    }

    java_information_list
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_java_information() {
        // println!(
        //     "{:?}",
        //     get_java_information(Path::new(r"C:\Program Files\Java\jdk-17.0.5"))
        // );
    }

    #[test]
    fn test_search_java_directory() {
        // println!("{:?}", search_java_directory(r"C:\Program Files\Java"));
    }

    #[test]
    fn test_find_system_java_versions_information() {
        println!(
            "{:#?}",
            find_system_java_versions_information(None::<Vec<&str>>)
        );
    }
}
