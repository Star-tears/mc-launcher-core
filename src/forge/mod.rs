use std::{fs, path::Path, process::Command};

use crate::{
    types::CallbackDict,
    utils::helper::{download_file, parse_maven_metadata},
};

pub fn run_forge_installer(
    version: &str,
    java: Option<impl AsRef<Path>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let forge_download_url=
        format!("https://files.minecraftforge.net/maven/net/minecraftforge/forge/{0}/forge-{0}-installer.jar",version);
    let temp_file_path = std::env::temp_dir().join(format!("forge-{}.tmp", rand::random::<u32>()));

    match download_file(
        &forge_download_url,
        &temp_file_path,
        None,
        false,
        None::<&Path>,
        None,
        &CallbackDict::default(),
    ) {
        Ok(v) => {
            if !v {
                return Err(format!("Version {} not found.", version).into());
            }
        }
        Err(e) => return Err(e),
    }

    let mut execute_name = "java";
    if let Some(java_path) = &java {
        execute_name = java_path.as_ref().to_str().unwrap_or("java");
    }
    let mut cmd = Command::new(execute_name);

    let _ = cmd.arg("-jar").arg(&temp_file_path).status();
    fs::remove_file(&temp_file_path)?;

    Ok(())
}

pub fn list_forge_versions() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let maven_metadata_url =
        "https://files.minecraftforge.net/maven/net/minecraftforge/forge/maven-metadata.xml";
    Ok(parse_maven_metadata(maven_metadata_url)?.versions)
}

pub fn forge_to_installed_version(forge_version: &str) -> Result<String, String> {
    match forge_version.split_once("-") {
        Some((vanilla_part, forge_part)) => Ok(format!("{}-forge-{}", vanilla_part, forge_part)),
        None => Err(format!("{} is not a valid forge version", forge_version)),
    }
}
