//! Minecraft version JSON data model.
//!
//! These structs represent the subset of Mojang and loader profile metadata
//! needed for installation and launch-command construction.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::rules::Rule;

/// Minecraft version or loader profile metadata.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VersionJson {
    /// Version or profile id.
    pub id: Option<String>,
    /// Parent profile id, if this profile inherits metadata.
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    /// Release type, such as `release` or `snapshot`.
    #[serde(default)]
    pub r#type: Option<String>,
    /// Java main class.
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    /// Minimum official launcher version.
    #[serde(rename = "minimumLauncherVersion", default)]
    pub minimum_launcher_version: Option<i32>,
    /// Asset index id.
    #[serde(default)]
    pub assets: Option<String>,
    /// Asset index download metadata.
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    /// Download metadata keyed by artifact role, such as `client`.
    #[serde(default)]
    pub downloads: HashMap<String, DownloadInfo>,
    /// Libraries required by the version.
    #[serde(default)]
    pub libraries: Vec<Library>,
    /// Modern JVM and game argument metadata.
    #[serde(default)]
    pub arguments: Arguments,
    /// Legacy game argument string used by older versions.
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    /// Java runtime metadata from Mojang.
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    /// Logging metadata keyed by logging type.
    #[serde(default)]
    pub logging: HashMap<String, LoggingConfig>,
    /// Client jar id override.
    pub jar: Option<String>,
    /// Version release timestamp.
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
    /// Version metadata timestamp.
    pub time: Option<String>,
    /// Mojang compliance level.
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
}

/// Modern argument sections.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct Arguments {
    /// Game arguments passed after the main class.
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    /// JVM arguments passed before the main class.
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

/// One argument entry from modern version metadata.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ArgumentValue {
    /// Plain argument string.
    String(String),
    /// Argument value gated by rules.
    Ruled {
        /// Rules that decide whether the value applies.
        #[serde(default)]
        rules: Vec<Rule>,
        /// String or string-list value to append when rules match.
        value: StringOrVec,
    },
}

/// Argument payload that may be a single string or a list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum StringOrVec {
    /// One argument value.
    String(String),
    /// Multiple argument values.
    Vec(Vec<String>),
}

/// Asset index download metadata.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssetIndex {
    /// Asset index id.
    pub id: String,
    /// SHA-1 checksum.
    pub sha1: String,
    /// Compressed index size in bytes.
    pub size: i64,
    /// Total referenced asset size in bytes.
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    /// Download URL.
    pub url: String,
}

/// Download metadata for one file.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DownloadInfo {
    /// SHA-1 checksum.
    pub sha1: String,
    /// File size in bytes.
    pub size: i64,
    /// Download URL.
    pub url: String,
}

/// Mojang Java runtime requirement.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JavaVersion {
    /// Runtime component name.
    pub component: String,
    /// Java major version.
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

/// Library entry from version metadata.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Library {
    /// Maven coordinate.
    pub name: String,
    /// Optional repository URL for legacy metadata.
    pub url: Option<String>,
    /// Rules that decide whether this library applies.
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Artifact and classifier downloads.
    pub downloads: Option<LibraryDownloads>,
    /// Native classifier templates by Minecraft OS name.
    pub natives: Option<HashMap<String, String>>,
    /// Extraction options, including excluded prefixes.
    pub extract: Option<HashMap<String, Vec<String>>>,
}

/// Download metadata attached to a library.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LibraryDownloads {
    /// Main artifact download.
    pub artifact: Option<LibraryArtifact>,
    /// Classifier artifact downloads keyed by classifier.
    #[serde(default)]
    pub classifiers: HashMap<String, LibraryArtifact>,
}

/// One library artifact download.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LibraryArtifact {
    /// Relative path under the local Maven repository.
    pub path: String,
    /// Download URL.
    pub url: String,
    /// SHA-1 checksum.
    pub sha1: String,
    /// File size in bytes.
    pub size: i64,
}

/// Logging configuration for a version.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingConfig {
    /// JVM argument template for enabling logging.
    pub argument: String,
    /// Logging configuration file metadata.
    pub file: LoggingFile,
    /// Logging type.
    pub r#type: String,
}

/// Logging configuration file metadata.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingFile {
    /// File id.
    pub id: String,
    /// SHA-1 checksum.
    pub sha1: String,
    /// File size in bytes.
    pub size: i64,
    /// Download URL.
    pub url: String,
}

impl VersionJson {
    /// Merges child profile metadata into a parent version.
    ///
    /// Loader profiles commonly inherit from a vanilla version. This method
    /// keeps parent defaults and lets child values override or extend them.
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
