//! NeoForge metadata helpers.

use crate::{
    loader::{forge::MavenMetadata, LoaderKind},
    net::http,
    LauncherError, Result,
};

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

/// Returns the newest advertised NeoForge version for a Minecraft version.
///
/// NeoForge maps Minecraft `1.x.y` releases to loader versions beginning with
/// `x.y.`. A release without a patch component, such as `1.21`, maps to
/// `21.0.`. Minecraft's calendar-versioned releases retain their full version,
/// so `26.1.2` maps to `26.1.2.`.
///
/// # Errors
///
/// Returns [`crate::LauncherError::LoaderVersionNotFound`] when the Minecraft
/// version is unsupported or the metadata has no matching NeoForge version.
pub fn latest_for_minecraft<'a>(
    versions: &'a [String],
    minecraft_version: &str,
) -> Result<&'a str> {
    let prefix = neoforge_version_prefix(minecraft_version);
    prefix
        .as_deref()
        .and_then(|prefix| versions.iter().rfind(|version| version.starts_with(prefix)))
        .map(String::as_str)
        .ok_or_else(|| LauncherError::LoaderVersionNotFound {
            loader: LoaderKind::NeoForge,
            version: format!("latest for Minecraft {minecraft_version}"),
        })
}

fn neoforge_version_prefix(minecraft_version: &str) -> Option<String> {
    if let Some(legacy) = minecraft_version.strip_prefix("1.") {
        let mut components = legacy.split('.');
        let release = components.next()?;
        let patch = components.next().unwrap_or("0");

        if components.next().is_some() || !numeric_components(&[release, patch]) {
            return None;
        }

        return Some(format!("{release}.{patch}."));
    }

    let mut components = minecraft_version.split('.');
    let year = components.next()?;
    let release = components.next()?;
    let patch = components.next().unwrap_or("0");

    if components.next().is_some()
        || year.parse::<u32>().ok()? < 26
        || !numeric_components(&[year, release, patch])
    {
        return None;
    }

    Some(format!("{year}.{release}.{patch}."))
}

fn numeric_components(components: &[&str]) -> bool {
    components.iter().all(|component| {
        !component.is_empty()
            && component
                .chars()
                .all(|character| character.is_ascii_digit())
    })
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
