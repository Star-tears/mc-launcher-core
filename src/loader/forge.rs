use regex::Regex;

use crate::{net::http, LauncherError, Result};

const FORGE_METADATA_URL: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenMetadata {
    pub latest: String,
    pub release: String,
    pub versions: Vec<String>,
}

pub fn parse_maven_metadata(xml: &str) -> Result<MavenMetadata> {
    let latest = capture_one(xml, r"<latest>(.*?)</latest>", "latest")?;
    let release = capture_one(xml, r"<release>(.*?)</release>", "release")?;
    let version_re =
        Regex::new(r"<version>(.*?)</version>").map_err(|err| LauncherError::Other {
            message: err.to_string(),
        })?;
    let versions = version_re
        .captures_iter(xml)
        .filter_map(|captures| captures.get(1).map(|m| m.as_str().to_string()))
        .collect();
    Ok(MavenMetadata {
        latest,
        release,
        versions,
    })
}

fn capture_one(xml: &str, pattern: &str, field: &str) -> Result<String> {
    let re = Regex::new(pattern).map_err(|err| LauncherError::Other {
        message: err.to_string(),
    })?;
    re.captures(xml)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| LauncherError::MissingField {
            context: "maven metadata".to_string(),
            field: field.to_string(),
        })
}

pub fn list_forge_versions() -> Result<Vec<String>> {
    Ok(parse_maven_metadata(&http::get_text(FORGE_METADATA_URL)?)?.versions)
}

pub fn forge_installed_version_id(forge_version: &str) -> Result<String> {
    let Some((minecraft, forge)) = forge_version.split_once('-') else {
        return Err(LauncherError::InvalidVersionId {
            id: forge_version.to_string(),
        });
    };
    Ok(format!("{minecraft}-forge-{forge}"))
}

pub fn installer_url(forge_version: &str) -> String {
    format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{0}/forge-{0}-installer.jar",
        forge_version
    )
}
