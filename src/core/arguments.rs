use std::{collections::HashMap, path::Path};

use crate::{
    account::Account,
    core::{
        rules::{evaluate_rules, FeatureSet},
        version::{ArgumentValue, StringOrVec, VersionJson},
    },
    platform::Platform,
};

#[derive(Debug, Clone)]
pub struct ArgumentContext<'a> {
    pub minecraft_dir: &'a Path,
    pub natives_dir: &'a Path,
    pub game_dir: &'a Path,
    pub version: &'a VersionJson,
    pub account: &'a Account,
    pub classpath: &'a str,
    pub launcher_name: &'a str,
    pub launcher_version: &'a str,
    pub version_type: &'a str,
    pub assets_index: &'a str,
    pub extra: HashMap<&'a str, &'a str>,
}

pub fn evaluate_arguments(
    values: &[ArgumentValue],
    context: &ArgumentContext<'_>,
    features: &FeatureSet,
    platform: Platform,
) -> Vec<String> {
    let mut args = Vec::new();
    for value in values {
        match value {
            ArgumentValue::String(raw) => args.push(replace_placeholders(raw, context)),
            ArgumentValue::Ruled { rules, value } => {
                if evaluate_rules(rules, platform, features) {
                    match value {
                        StringOrVec::String(raw) => args.push(replace_placeholders(raw, context)),
                        StringOrVec::Vec(raw_values) => {
                            args.extend(
                                raw_values
                                    .iter()
                                    .map(|raw| replace_placeholders(raw, context)),
                            );
                        }
                    }
                }
            }
        }
    }
    args
}

pub fn replace_placeholders(raw: &str, context: &ArgumentContext<'_>) -> String {
    let version_name = context.version.id.as_deref().unwrap_or_default();
    let assets_root = context.minecraft_dir.join("assets");
    let library_directory = context.minecraft_dir.join("libraries");
    let game_assets = assets_root.join("virtual").join("legacy");

    let mut value = raw.to_string();
    let replacements = [
        (
            "${natives_directory}",
            context.natives_dir.to_string_lossy().to_string(),
        ),
        ("${launcher_name}", context.launcher_name.to_string()),
        ("${launcher_version}", context.launcher_version.to_string()),
        ("${classpath}", context.classpath.to_string()),
        (
            "${auth_player_name}",
            context.account.username().to_string(),
        ),
        ("${version_name}", version_name.to_string()),
        (
            "${game_directory}",
            context.game_dir.to_string_lossy().to_string(),
        ),
        ("${assets_root}", assets_root.to_string_lossy().to_string()),
        ("${assets_index_name}", context.assets_index.to_string()),
        ("${auth_uuid}", context.account.uuid().to_string()),
        (
            "${auth_access_token}",
            context.account.access_token().to_string(),
        ),
        ("${user_type}", "msa".to_string()),
        ("${version_type}", context.version_type.to_string()),
        ("${user_properties}", "{}".to_string()),
        ("${game_assets}", game_assets.to_string_lossy().to_string()),
        (
            "${auth_session}",
            context.account.access_token().to_string(),
        ),
        (
            "${library_directory}",
            library_directory.to_string_lossy().to_string(),
        ),
        ("${classpath_separator}", classpath_separator().to_string()),
    ];

    for (key, replacement) in replacements {
        value = value.replace(key, &replacement);
    }
    for (key, replacement) in &context.extra {
        value = value.replace(key, replacement);
    }
    value
}

pub fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}
