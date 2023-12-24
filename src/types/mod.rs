use chrono::{DateTime, Utc};

struct MinecraftOptions {
    username: String,
    uuid: String,
    token: String,
    executable_path: String,
    default_executable_path: String,
    jvm_arguments: Vec<String>,
    launcher_name: String,
    launcher_version: String,
    game_directory: String,
    demo: bool,
    custom_resolution: bool,
    resolution_width: String,
    resolution_height: String,
    server: String,
    port: String,
    natives_directory: String,
    enable_logging_config: bool,
    disable_multiplayer: bool,
    disable_chat: bool,
    quick_play_path: Option<String>,
    quick_play_singleplayer: Option<String>,
    quick_play_multiplayer: Option<String>,
    quick_play_realms: Option<String>,
}

struct CallbackDict {
    set_status: Option<fn(String)>,
    set_progress: Option<fn(i32)>,
    set_max: Option<fn(i32)>,
}

struct LatestMinecraftVersions {
    release: String,
    snapshot: String,
}

struct MinecraftVersionInfo {
    id: String,
    type_: String,
    release_time: DateTime<Utc>,
    compliance_level: i32,
}

// fabric

struct FabricMinecraftVersion {
    version: String,
    stable: bool,
}

struct FabricLoader {
    separator: String,
    build: i32,
    maven: String,
    version: String,
    stable: bool,
}

// quilt

struct QuiltMinecraftVersion {
    version: String,
    stable: bool,
}

struct QuiltLoader {
    separator: String,
    build: i32,
    maven: String,
    version: String,
}

struct Image {
    content_type: String,
    image_url: String,
    alt: Option<String>,
    video_url: Option<String>,
    video_type: Option<String>,
    video_provider: Option<String>,
    video_id: Option<String>,
    linkurl: Option<String>,
    background_color: Option<String>,
}

struct Tile {
    sub_header: String,
    image: Image,
    tile_size: String,
    title: String,
}

struct Article {
    default_tile: Tile,
    article_lang: String,
    primary_category: String,
    categories: Vec<String>,
    article_url: String,
    publish_date: String,
    tags: Vec<String>,
    preferred_tile: Option<Tile>,
}

struct Articles {
    article_grid: Vec<Article>,
    article_count: i32,
}

// java_utils
struct JavaInformation {
    path: String,
    name: String,
    version: String,
    java_path: String,
    javaw_path: Option<String>,
    is_64bit: bool,
    openjdk: bool,
}

// vanilla_launcher
struct VanillaLauncherProfileResolution {
    height: i32,
    width: i32,
}

struct VanillaLauncherProfile {
    name: String,
    version: Option<String>,
    version_type: String,
    game_directory: Option<String>,
    java_executable: Option<String>,
    java_arguments: Option<Vec<String>>,
    custom_resolution: Option<VanillaLauncherProfileResolution>,
}

// mrpack
struct MrpackInformation {
    name: String,
    summary: String,
    version_id: String,
    format_version: i32,
    minecraft_version: String,
    optional_files: Vec<String>,
}

struct MrpackInstallOptions {
    optional_files: Vec<String>,
    skip_dependencies_install: bool,
}

// runtime
struct JvmRuntimeInformation {
    name: String,
    released: DateTime<Utc>,
}
