use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonRule {
    pub action: String,
    pub os: Option<HashMap<String, String>>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringAndVecStringValue {
    StringValue(String),
    VecStringValue(Vec<String>),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonArgumentRule {
    pub compatibility_rules: Option<Vec<ClientJsonRule>>,
    pub rules: Option<Vec<ClientJsonRule>>,
    pub value: Option<StringAndVecStringValue>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    #[serde(rename = "totalSize")]
    pub total_size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonDownloads {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonJavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonLibraryDownloadsArtifact {
    pub path: String,
    pub url: String,
    pub sha1: String,
    pub size: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonLibraryDownloads {
    pub artifact: Option<ClientJsonLibraryDownloadsArtifact>,
    /// keys: {"javadoc", "natives-linux", "natives-macos", "natives-windows", "sources"}
    pub classifiers: Option<HashMap<String, ClientJsonLibraryDownloadsArtifact>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonLibrary {
    pub name: Option<String>,
    pub downloads: Option<ClientJsonLibraryDownloads>,
    /// keys: {"exclude"}
    pub extract: Option<HashMap<String, Vec<String>>>,
    pub rules: Option<Vec<ClientJsonRule>>,
    /// keys: {"linux", "osx", "windows"}
    pub natives: Option<HashMap<String, String>>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonLoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJsonLogging {
    pub argument: String,
    pub file: ClientJsonLoggingFile,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringAndClientJsonArgumentRuleValue {
    StringValue(String),
    ClientJsonArgumentRuleValue(ClientJsonArgumentRule),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientJson {
    pub id: Option<String>,
    pub assets: Option<String>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i32,
    pub jar: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
    pub r#type: Option<String>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    pub libraries: Option<Vec<ClientJsonLibrary>>,
    pub arguments: Option<HashMap<String, Vec<StringAndClientJsonArgumentRuleValue>>>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<ClientJsonAssetIndex>,
    pub downloads: Option<HashMap<String, ClientJsonDownloads>>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<ClientJsonJavaVersion>,
    pub logging: Option<HashMap<String, ClientJsonLogging>>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
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

// impl
impl ClientJson {
    pub fn merge(&mut self, other: &ClientJson) {
        self.id = other.id.clone();
        self.assets = other.assets.clone();
        if let Some(minecraft_arguments) = &other.minecraft_arguments {
            self.minecraft_arguments = Some(minecraft_arguments.clone());
        }
        self.time = other.time.clone();
        self.minimum_launcher_version = other.minimum_launcher_version.clone();
        if let Some(jar) = &other.jar {
            self.jar = Some(jar.clone());
        }
        self.main_class = other.main_class.clone();
        self.release_time = other.release_time.clone();
        self.r#type = other.r#type.clone();
        if let Some(inherits_from) = &other.inherits_from {
            self.inherits_from = Some(inherits_from.clone());
        }
        if let Some(libraries) = &other.libraries {
            let mut tmpv = self.libraries.clone().unwrap_or(vec![]).clone();
            tmpv.extend_from_slice(&libraries);
            self.libraries = Some(tmpv);
        }

        if let Some(arguments) = &other.arguments {
            if let Some(self_arguments) = &mut self.arguments {
                for (key, value) in arguments.iter() {
                    if let Some(self_value) = self_arguments.get_mut(key) {
                        self_value.extend_from_slice(value);
                    } else {
                        self_arguments.insert(key.clone(), value.clone());
                    }
                }
            } else {
                self.arguments = Some(arguments.clone());
            }
        }
        if let Some(asset_index) = &other.asset_index {
            self.asset_index = Some(asset_index.clone());
        }
        if let Some(java_version) = &other.java_version {
            self.java_version = Some(java_version.clone());
        }
        if let Some(compliance_level) = &other.compliance_level {
            self.compliance_level = Some(compliance_level.clone());
        }
    }
}
