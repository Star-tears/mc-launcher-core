use mc_launcher_core::{
    core::{
        rules::{evaluate_rules, FeatureSet, Rule, RuleAction, RuleOs},
        version::VersionJson,
    },
    platform::{Arch, Os, Platform},
};

#[test]
fn allows_matching_os_rule() {
    let rules = vec![Rule {
        action: RuleAction::Allow,
        os: Some(RuleOs {
            name: Some("osx".to_string()),
            arch: None,
            version: None,
        }),
        features: None,
    }];

    assert!(evaluate_rules(
        &rules,
        Platform {
            os: Os::MacOs,
            arch: Arch::Aarch64,
        },
        &FeatureSet::default()
    ));
}

#[test]
fn rejects_non_matching_os_rule() {
    let rules = vec![Rule {
        action: RuleAction::Allow,
        os: Some(RuleOs {
            name: Some("windows".to_string()),
            arch: None,
            version: None,
        }),
        features: None,
    }];

    assert!(!evaluate_rules(
        &rules,
        Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        },
        &FeatureSet::default()
    ));
}

#[test]
fn child_version_overrides_main_class_and_extends_libraries() {
    let parent: VersionJson = serde_json::from_str(
        r#"{
            "id":"1.20.4",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "minimumLauncherVersion":21,
            "libraries":[{"name":"com.example:parent:1.0"}],
            "arguments":{"game":["--username","${auth_player_name}"],"jvm":["-cp","${classpath}"]}
        }"#,
    )
    .unwrap();

    let child: VersionJson = serde_json::from_str(
        r#"{
            "id":"fabric-loader-0.15.7-1.20.4",
            "inheritsFrom":"1.20.4",
            "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
            "minimumLauncherVersion":21,
            "libraries":[{"name":"net.fabricmc:fabric-loader:0.15.7"}],
            "arguments":{"game":[],"jvm":["-DFabricMcEmu= net.minecraft.client.main.Main "]}
        }"#,
    )
    .unwrap();

    let merged = parent.merge_child(&child);
    assert_eq!(
        merged.main_class.as_deref(),
        Some("net.fabricmc.loader.impl.launch.knot.KnotClient")
    );
    assert_eq!(merged.libraries.len(), 2);
    assert_eq!(merged.arguments.jvm.len(), 3);
}

#[test]
fn inherited_loader_version_uses_parent_client_jar() {
    let parent: VersionJson = serde_json::from_str(
        r#"{
            "id":"1.20.1",
            "mainClass":"net.minecraft.client.main.Main",
            "arguments":{"game":[],"jvm":[]}
        }"#,
    )
    .unwrap();
    let child: VersionJson = serde_json::from_str(
        r#"{
            "id":"fabric-loader-0.19.2-1.20.1",
            "inheritsFrom":"1.20.1",
            "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
            "arguments":{"game":[],"jvm":[]}
        }"#,
    )
    .unwrap();

    let merged = parent.merge_child(&child);

    assert_eq!(merged.id.as_deref(), Some("fabric-loader-0.19.2-1.20.1"));
    assert_eq!(merged.jar.as_deref(), Some("1.20.1"));
}
