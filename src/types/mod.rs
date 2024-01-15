use chrono::{DateTime, Utc};
use serde::Deserialize;

pub mod exceptions_types;
pub mod forge_types;
pub mod helper_types;
pub mod install_types;
pub mod microsoft_types;
pub mod mrpack_types;
pub mod runtime_types;
pub mod shared_types;
pub mod vanilla_launcher_types;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct MinecraftOptions {
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub token: Option<String>,
    pub executable_path: Option<String>,
    pub default_executable_path: Option<String>,
    pub jvm_arguments: Option<Vec<String>>,
    pub launcher_name: Option<String>,
    pub launcher_version: Option<String>,
    pub game_directory: Option<String>,
    pub demo: Option<bool>,
    pub custom_resolution: Option<bool>,
    pub resolution_width: Option<String>,
    pub resolution_height: Option<String>,
    pub server: Option<String>,
    pub port: Option<String>,
    pub natives_directory: Option<String>,
    pub enable_logging_config: Option<bool>,
    pub disable_multiplayer: Option<bool>,
    pub disable_chat: Option<bool>,
    pub quick_play_path: Option<String>,
    pub quick_play_singleplayer: Option<String>,
    pub quick_play_multiplayer: Option<String>,
    pub quick_play_realms: Option<String>,
}

#[derive(Clone, Copy, Default)]
pub struct CallbackDict {
    pub set_status: Option<fn(String)>,
    pub set_progress: Option<fn(i32)>,
    pub set_max: Option<fn(i32)>,
}

#[derive(Debug)]
pub struct LatestMinecraftVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone)]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub r#type: String,
    pub release_time: DateTime<Utc>,
    pub compliance_level: i32,
}

// fabric
pub struct FabricMinecraftVersion {
    pub version: String,
    pub stable: bool,
}

pub struct FabricLoader {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

// quilt
pub struct QuiltMinecraftVersion {
    pub version: String,
    pub stable: bool,
}

pub struct QuiltLoader {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
}

// minecraft news
pub struct MinecraftNewsOptions {
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub content_type: Option<String>,
    #[serde(rename = "imageURL")]
    pub image_url: Option<String>,
    pub alt: Option<String>,
    #[serde(rename = "videoURL")]
    pub video_url: Option<String>,
    #[serde(rename = "videoType")]
    pub video_type: Option<String>,
    #[serde(rename = "videoProvider")]
    pub video_provider: Option<String>,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    pub linkurl: Option<String>,
    pub background_color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    pub sub_header: String,
    pub image: Image,
    pub tile_size: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct Article {
    pub default_tile: Tile,
    #[serde(rename = "articleLang")]
    pub article_lang: String,
    pub primary_category: String,
    pub categories: Vec<String>,
    pub article_url: String,
    pub publish_date: String,
    pub tags: Vec<String>,
    pub preferred_tile: Option<Tile>,
}

#[derive(Debug, Deserialize)]
pub struct Articles {
    pub article_grid: Vec<Article>,
    pub article_count: i32,
}

// java_utils
#[derive(Debug)]
pub struct JavaInformation {
    pub path: String,
    pub name: String,
    pub version: String,
    pub java_path: String,
    pub javaw_path: Option<String>,
    pub is_64bit: bool,
    pub openjdk: bool,
}

// vanilla_launcher
#[derive(Debug, Deserialize)]
pub struct VanillaLauncherProfileResolution {
    pub height: i32,
    pub width: i32,
}

#[derive(Debug, Deserialize)]
pub struct VanillaLauncherProfile {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "versionType")]
    pub version_type: Option<String>,
    #[serde(rename = "gameDirectory")]
    pub game_directory: Option<String>,
    #[serde(rename = "javaExecutable")]
    pub java_executable: Option<String>,
    #[serde(rename = "javaArguments")]
    pub java_arguments: Option<Vec<String>>,
    #[serde(rename = "customResolution")]
    pub custom_resolution: Option<VanillaLauncherProfileResolution>,
}

// mrpack
pub struct MrpackInformation {
    pub name: String,
    pub summary: String,
    pub version_id: String,
    pub format_version: i32,
    pub minecraft_version: String,
    pub optional_files: Vec<String>,
}

pub struct MrpackInstallOptions {
    pub optional_files: Option<Vec<String>>,
    pub skip_dependencies_install: Option<bool>,
}

// runtime
pub struct JvmRuntimeInformation {
    pub name: String,
    pub released: DateTime<Utc>,
}

pub struct VersionRuntimeInformation {
    pub name: String,
    pub java_major_version: i32,
}

// impl
impl JavaInformation {
    pub fn new(
        path: &str,
        name: &str,
        version: &str,
        is_64bit: bool,
        openjdk: bool,
        java_path: &str,
        javaw_path: Option<String>,
    ) -> Self {
        JavaInformation {
            path: path.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            is_64bit,
            openjdk,
            java_path: java_path.to_string(),
            javaw_path,
        }
    }
}

// impl
impl MinecraftOptions {
    pub fn new(username: String, uuid: String, token: String) -> Self {
        Self {
            username: Some(username),
            uuid: Some(uuid),
            token: Some(token),
            resolution_width: None,
            resolution_height: None,
            executable_path: None,
            default_executable_path: None,
            jvm_arguments: None,
            launcher_name: None,
            launcher_version: None,
            game_directory: None,
            demo: None,
            custom_resolution: None,
            server: None,
            port: None,
            natives_directory: None,
            enable_logging_config: None,
            disable_multiplayer: None,
            disable_chat: None,
            quick_play_path: None,
            quick_play_singleplayer: None,
            quick_play_multiplayer: None,
            quick_play_realms: None,
        }
    }
}

impl Default for MinecraftNewsOptions {
    fn default() -> Self {
        MinecraftNewsOptions { page_size: 20 }
    }
}
