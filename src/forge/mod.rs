pub fn forge_to_installed_version(forge_version: &str) -> Result<String, String> {
    match forge_version.split_once("-") {
        Some((vanilla_part, forge_part)) => Ok(format!("{}-forge-{}", vanilla_part, forge_part)),
        None => Err(format!("{} is not a valid forge version", forge_version)),
    }
}
