use std::path::Path;

use mc_launcher_core::{
    core::{classpath::classpath_entries_for_platform, version::VersionJson},
    platform::Platform,
};

fn forge_profile(main_class: &str) -> VersionJson {
    serde_json::from_str(&format!(
        r#"{{
            "id":"forge-profile",
            "jar":"1.12.2",
            "mainClass":"{main_class}",
            "libraries":[{{"name":"net.minecraftforge:forge:1.12.2-14.23.5.2860"}}]
        }}"#
    ))
    .unwrap()
}

#[test]
fn legacy_forge_profile_keeps_parent_client_jar_on_classpath() {
    let version = forge_profile("net.minecraft.launchwrapper.Launch");
    let entries = classpath_entries_for_platform(&version, "/tmp/mc", Platform::current()).unwrap();

    assert!(entries
        .iter()
        .any(|entry| entry.ends_with(Path::new("versions/1.12.2/1.12.2.jar"))));
}

#[test]
fn forge_bootstrap_profile_omits_parent_client_jar_from_classpath() {
    let version = forge_profile("net.minecraftforge.bootstrap.ForgeBootstrap");
    let entries = classpath_entries_for_platform(&version, "/tmp/mc", Platform::current()).unwrap();

    assert!(!entries
        .iter()
        .any(|entry| entry.ends_with(Path::new("versions/1.12.2/1.12.2.jar"))));
}

#[test]
fn modlauncher_bootstrap_profile_omits_parent_client_jar_from_classpath() {
    let version = forge_profile("cpw.mods.bootstraplauncher.BootstrapLauncher");
    let entries = classpath_entries_for_platform(&version, "/tmp/mc", Platform::current()).unwrap();

    assert!(!entries
        .iter()
        .any(|entry| entry.ends_with(Path::new("versions/1.12.2/1.12.2.jar"))));
}

#[test]
fn duplicate_library_paths_are_emitted_once() {
    let version: VersionJson = serde_json::from_str(
        r#"{
            "id":"neoforge-21.1.244",
            "mainClass":"cpw.mods.bootstraplauncher.BootstrapLauncher",
            "libraries":[
                {
                    "name":"com.google.code.gson:gson:2.10.1",
                    "downloads":{"artifact":{"path":"com/google/code/gson/gson/2.10.1/gson-2.10.1.jar","url":"","sha1":"","size":1}}
                },
                {
                    "name":"com.google.code.gson:gson:2.10.1",
                    "downloads":{"artifact":{"path":"com/google/code/gson/gson/2.10.1/gson-2.10.1.jar","url":"","sha1":"","size":1}}
                }
            ]
        }"#,
    )
    .unwrap();

    let entries = classpath_entries_for_platform(&version, "/tmp/mc", Platform::current()).unwrap();

    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.ends_with(Path::new("gson-2.10.1.jar")))
            .count(),
        1
    );
}
