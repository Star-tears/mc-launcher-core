//! NeoForge metadata helpers.

use crate::{loader::forge::MavenMetadata, net::http, Result};

const NEOFORGE_METADATA_URL: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

/// Parses NeoForge Maven metadata XML.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if required metadata fields are missing.
pub fn parse_maven_metadata(xml: &str) -> Result<MavenMetadata> {
    crate::loader::forge::parse_maven_metadata(xml)
}

/// Fetches all NeoForge versions from the NeoForge Maven repository.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the metadata request or parsing fails.
pub fn list_neoforge_versions() -> Result<Vec<String>> {
    Ok(parse_maven_metadata(&http::get_text(NEOFORGE_METADATA_URL)?)?.versions)
}

/// Returns the installed NeoForge profile id.
pub fn neoforge_installed_version_id(_minecraft_version: &str, neoforge_version: &str) -> String {
    format!("neoforge-{neoforge_version}")
}

/// Returns the NeoForge installer jar URL for a NeoForge version.
pub fn installer_url(neoforge_version: &str) -> String {
    format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{0}/neoforge-{0}-installer.jar",
        neoforge_version
    )
}
