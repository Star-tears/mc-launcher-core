use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
pub struct VanillaLauncherProfilesJsonProfile {
    pub created: Option<String>,
    pub game_dir: Option<String>,
    pub icon: Option<String>,
    pub java_args: Option<String>,
    pub java_dir: Option<String>,
    pub last_used: Option<String>,
    pub last_version_id: Option<String>,
    pub name: Option<String>,
    pub resolution: Option<HashMap<String, i32>>,
    pub type_: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct VanillaLauncherProfilesJson {
    pub profiles: HashMap<String, VanillaLauncherProfilesJsonProfile>,
    pub version: i32,
}
