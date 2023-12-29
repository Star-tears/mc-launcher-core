use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ClientJsonRule {
    pub action: String,
    pub os: Option<HashMap<String, String>>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StringAndVecStringValue {
    StringValue(String),
    VecStringValue(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonArgumentRule {
    pub compatibility_rules: Option<Vec<ClientJsonRule>>,
    pub rules: Option<Vec<ClientJsonRule>>,
    pub value: Option<StringAndVecStringValue>,
}

#[derive(Debug, Deserialize)]
pub struct ClientJsonAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    #[serde(rename = "totalSize")]
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
    #[serde(rename = "majorVersion")]
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
    pub downloads: Option<ClientJsonLibraryDownloads>,
    pub extract: Option<HashMap<String, Vec<String>>>,
    pub rules: Option<Vec<ClientJsonRule>>,
    pub natives: Option<HashMap<String, String>>,
    pub url: Option<String>,
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
#[serde(untagged)]
pub enum StringAndClientJsonArgumentRuleValue {
    StringValue(String),
    ClientJsonArgumentRuleValue(ClientJsonArgumentRule),
}

#[derive(Debug, Deserialize)]
pub struct ClientJson {
    pub id: String,
    pub jar: Option<String>,
    pub arguments: HashMap<String, Vec<StringAndClientJsonArgumentRuleValue>>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: ClientJsonAssetIndex,
    pub assets: String,
    pub downloads: HashMap<String, ClientJsonDownloads>,
    #[serde(rename = "javaVersion")]
    pub java_version: ClientJsonJavaVersion,
    pub libraries: Vec<ClientJsonLibrary>,
    pub logging: HashMap<String, ClientJsonLogging>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i32,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub time: String,
    pub r#type: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
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
