use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ClientJsonRule {
    pub action: String,
    pub os: HashMap<String, String>,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonArgumentRule {
    pub compatibility_rules: Option<Vec<ClientJsonRule>>,
    pub rules: Option<Vec<ClientJsonRule>>,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub total_size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonDownloads {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonJavaVersion {
    pub component: String,
    pub major_version: i32,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonLibraryDownloadsArtifact {
    pub path: String,
    pub url: String,
    pub sha1: String,
    pub size: i32,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonLibraryDownloads {
    pub artifact: Option<ClientJsonLibraryDownloadsArtifact>,
    pub classifiers: Option<HashMap<String, ClientJsonLibraryDownloadsArtifact>>,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonLibrary {
    pub name: String,
    pub downloads: ClientJsonLibraryDownloads,
    pub extract: Option<HashMap<String, Vec<String>>>,
    pub rules: Option<Vec<ClientJsonRule>>,
    pub natives: Option<HashMap<String, String>>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonLoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonLogging {
    pub argument: String,
    pub file: ClientJsonLoggingFile,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientJson {
    pub id: String,
    pub jar: String,
    pub arguments: HashMap<String, Vec<String>>,
    pub minecraft_arguments: String,
    pub asset_index: ClientJsonAssetIndex,
    pub assets: String,
    pub downloads: HashMap<String, ClientJsonDownloads>,
    pub java_version: ClientJsonJavaVersion,
    pub libraries: Vec<ClientJsonLibrary>,
    pub logging: HashMap<String, ClientJsonLogging>,
    pub main_class: String,
    pub minimum_launcher_version: i32,
    pub release_time: String,
    pub time: String,
    pub r#type: String,
    pub compliance_level: i32,
    pub inherits_from: String,
}

// need same as json
#[derive(Debug, Deserialize)]
pub struct VersionListManifestJsonVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,
}

#[derive(Debug, Deserialize)]
pub struct VersionListManifestJson {
    pub latest: HashMap<String, String>,
    pub versions: Vec<VersionListManifestJsonVersion>,
}
