//! Argument rule evaluation and placeholder replacement.

use std::{collections::HashMap, path::Path};

use crate::{
    account::Account,
    core::{
        rules::{evaluate_rules, FeatureSet},
        version::{ArgumentValue, StringOrVec, VersionJson},
    },
    platform::Platform,
};

/// Values used when replacing placeholders in Minecraft argument templates.
#[derive(Debug, Clone)]
pub struct ArgumentContext<'a> {
    /// Root Minecraft directory.
    pub minecraft_dir: &'a Path,
    /// Directory containing extracted native libraries.
    pub natives_dir: &'a Path,
    /// Game directory passed to Minecraft.
    pub game_dir: &'a Path,
    /// Version metadata being launched.
    pub version: &'a VersionJson,
    /// Account used for auth placeholders.
    pub account: &'a Account,
    /// Platform classpath string.
    pub classpath: &'a str,
    /// Launcher name placeholder value.
    pub launcher_name: &'a str,
    /// Launcher version placeholder value.
    pub launcher_version: &'a str,
    /// Version type placeholder value.
    pub version_type: &'a str,
    /// Asset index placeholder value.
    pub assets_index: &'a str,
    /// Additional placeholder replacements.
    pub extra: HashMap<&'a str, &'a str>,
}

/// Evaluates argument entries and filters ruled values for a platform.
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

/// Replaces known Minecraft placeholders in a single argument string.
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
        ("${clientid}", String::new()),
        ("${auth_xuid}", String::new()),
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

/// Returns the classpath separator for the current platform.
pub fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}
