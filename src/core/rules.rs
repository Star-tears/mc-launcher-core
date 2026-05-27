use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::platform::{Arch, Platform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RuleOs {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<RuleOs>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FeatureSet {
    pub demo_user: bool,
    pub custom_resolution: bool,
    pub quick_play: bool,
    pub quick_play_singleplayer: bool,
    pub quick_play_multiplayer: bool,
    pub quick_play_realms: bool,
}

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
