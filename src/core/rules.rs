//! Minecraft rule evaluation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::platform::{Arch, Platform};

/// Rule action from Minecraft version metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Allow when the rule matches.
    Allow,
    /// Disallow when the rule matches.
    Disallow,
}

/// Operating-system selector in a rule.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RuleOs {
    /// Minecraft OS name such as `windows`, `osx`, or `linux`.
    pub name: Option<String>,
    /// Optional architecture selector.
    pub arch: Option<String>,
    /// Optional OS version regex from metadata.
    pub version: Option<String>,
}

/// One allow/disallow rule.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Rule {
    /// Action applied when the rule matches.
    pub action: RuleAction,
    /// Optional OS selector.
    pub os: Option<RuleOs>,
    /// Optional feature flags required by this rule.
    pub features: Option<HashMap<String, bool>>,
}

/// Runtime feature flags used by argument and library rules.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FeatureSet {
    /// Whether the launch is a demo-user launch.
    pub demo_user: bool,
    /// Whether a custom resolution is being requested.
    pub custom_resolution: bool,
    /// Whether quick play is supported.
    pub quick_play: bool,
    /// Whether quick play single-player is active.
    pub quick_play_singleplayer: bool,
    /// Whether quick play multiplayer is active.
    pub quick_play_multiplayer: bool,
    /// Whether quick play Realms is active.
    pub quick_play_realms: bool,
}

/// Evaluates Minecraft allow/disallow rules for a platform and feature set.
pub fn evaluate_rules(rules: &[Rule], platform: Platform, features: &FeatureSet) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule, platform, features) {
            allowed = rule.action == RuleAction::Allow;
        }
    }
    allowed
}

fn rule_matches(rule: &Rule, platform: Platform, features: &FeatureSet) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != platform.minecraft_os_name() {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            if arch == "x86" && platform.arch != Arch::X86 {
                return false;
            }
        }
    }

    if let Some(rule_features) = &rule.features {
        for (name, expected) in rule_features {
            let actual = match name.as_str() {
                "is_demo_user" => features.demo_user,
                "has_custom_resolution" => features.custom_resolution,
                "has_quick_plays_support" => features.quick_play,
                "is_quick_play_singleplayer" => features.quick_play_singleplayer,
                "is_quick_play_multiplayer" => features.quick_play_multiplayer,
                "is_quick_play_realms" => features.quick_play_realms,
                _ => false,
            };
            if actual != *expected {
                return false;
            }
        }
    }

    true
}
