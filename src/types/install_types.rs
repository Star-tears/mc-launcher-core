use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
pub struct AssetsJsonObject {
    pub hash: String,
    pub size: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct AssetsJson {
    pub objects: HashMap<String, AssetsJsonObject>,
}
