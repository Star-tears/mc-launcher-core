use uuid::Uuid;

use crate::types::MinecraftOptions;

pub fn get_offline_options(user_name: &str) -> MinecraftOptions {
    let user_uuid = Uuid::new_v4();
    MinecraftOptions::new(user_name.to_string(), user_uuid.to_string(), "".to_string())
}
