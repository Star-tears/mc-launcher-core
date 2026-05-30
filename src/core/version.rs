use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::rules::Rule;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VersionJson {
    pub id: Option<String>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "minimumLauncherVersion", default)]
    pub minimum_launcher_version: Option<i32>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    #[serde(default)]
    pub downloads: HashMap<String, DownloadInfo>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub arguments: Arguments,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub logging: HashMap<String, LoggingConfig>,
    pub jar: Option<String>,
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    Ruled {
        #[serde(default)]
        rules: Vec<Rule>,
        value: StringOrVec,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Library {
    pub name: String,
    pub url: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    #[serde(default)]
    pub classifiers: HashMap<String, LibraryArtifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LibraryArtifact {
    pub path: String,
    pub url: String,
    pub sha1: String,
    pub size: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingConfig {
    pub argument: String,
    pub file: LoggingFile,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

impl VersionJson {
    pub fn merge_child(mut self, child: &VersionJson) -> VersionJson {
        let inherited_parent = child.inherits_from.clone();
        self.id = child.id.clone().or(self.id);
        self.inherits_from = child.inherits_from.clone().or(self.inherits_from);
        self.r#type = child.r#type.clone().or(self.r#type);
        self.main_class = child.main_class.clone().or(self.main_class);
        self.minimum_launcher_version = child
            .minimum_launcher_version
            .or(self.minimum_launcher_version);
        self.assets = child.assets.clone().or(self.assets);
        self.asset_index = child.asset_index.clone().or(self.asset_index);
        self.downloads.extend(child.downloads.clone());
        self.libraries.extend(child.libraries.clone());
        self.arguments.game.extend(child.arguments.game.clone());
        self.arguments.jvm.extend(child.arguments.jvm.clone());
        self.minecraft_arguments = child
            .minecraft_arguments
            .clone()
            .or(self.minecraft_arguments);
        self.java_version = child.java_version.clone().or(self.java_version);
        self.logging.extend(child.logging.clone());
        self.jar = child.jar.clone().or(self.jar).or(inherited_parent);
        self.release_time = child.release_time.clone().or(self.release_time);
        self.time = child.time.clone().or(self.time);
        self.compliance_level = child.compliance_level.or(self.compliance_level);
        self
    }
}
