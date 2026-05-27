use crate::{loader::forge::MavenMetadata, net::http, Result};

const NEOFORGE_METADATA_URL: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

pub fn parse_maven_metadata(xml: &str) -> Result<MavenMetadata> {
    crate::loader::forge::parse_maven_metadata(xml)
}

pub fn list_neoforge_versions() -> Result<Vec<String>> {
    Ok(parse_maven_metadata(&http::get_text(NEOFORGE_METADATA_URL)?)?.versions)
}

pub fn neoforge_installed_version_id(_minecraft_version: &str, neoforge_version: &str) -> String {
    format!("neoforge-{neoforge_version}")
}

pub fn installer_url(neoforge_version: &str) -> String {
    format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{0}/neoforge-{0}-installer.jar",
        neoforge_version
    )
}
