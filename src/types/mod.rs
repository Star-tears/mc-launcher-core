use chrono::{DateTime, Utc};

pub mod forge_types;
pub mod helper_types;
pub mod install_types;
pub mod mrpack_types;
pub mod runtime_types;
pub mod shared_types;
pub mod vanilla_launcher_types;

pub struct MinecraftOptions {
    pub username: String,
    pub uuid: String,
    pub token: String,
    pub executable_path: String,
    pub default_executable_path: String,
    pub jvm_arguments: Vec<String>,
    pub launcher_name: String,
    pub launcher_version: String,
    pub game_directory: String,
    pub demo: bool,
    pub custom_resolution: bool,
    pub resolution_width: String,
    pub resolution_height: String,
    pub server: String,
    pub port: String,
    pub natives_directory: String,
    pub enable_logging_config: bool,
    pub disable_multiplayer: bool,
    pub disable_chat: bool,
    pub quick_play_path: Option<String>,
    pub quick_play_singleplayer: Option<String>,
    pub quick_play_multiplayer: Option<String>,
    pub quick_play_realms: Option<String>,
}

pub struct CallbackDict {
    pub set_status: Option<fn(String)>,
    pub set_progress: Option<fn(i32)>,
    pub set_max: Option<fn(i32)>,
}

pub struct LatestMinecraftVersions {
    pub release: String,
    pub snapshot: String,
}

pub struct MinecraftVersionInfo {
    pub id: String,
    pub type_: String,
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

pub struct Image {
    pub content_type: String,
    pub image_url: String,
    pub alt: Option<String>,
    pub video_url: Option<String>,
    pub video_type: Option<String>,
    pub video_provider: Option<String>,
    pub video_id: Option<String>,
    pub linkurl: Option<String>,
    pub background_color: Option<String>,
}

pub struct Tile {
    pub sub_header: String,
    pub image: Image,
    pub tile_size: String,
    pub title: String,
}

pub struct Article {
    pub default_tile: Tile,
    pub article_lang: String,
    pub primary_category: String,
    pub categories: Vec<String>,
    pub article_url: String,
    pub publish_date: String,
    pub tags: Vec<String>,
    pub preferred_tile: Option<Tile>,
}

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
pub struct VanillaLauncherProfileResolution {
    pub height: i32,
    pub width: i32,
}

pub struct VanillaLauncherProfile {
    pub name: String,
    pub version: Option<String>,
    pub version_type: String,
    pub game_directory: Option<String>,
    pub java_executable: Option<String>,
    pub java_arguments: Option<Vec<String>>,
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
    pub optional_files: Vec<String>,
    pub skip_dependencies_install: bool,
}

// runtime
pub struct JvmRuntimeInformation {
    pub name: String,
    pub released: DateTime<Utc>,
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
