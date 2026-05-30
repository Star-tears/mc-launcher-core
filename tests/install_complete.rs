use std::{fs::File, io::Write};

use mc_launcher_core::{
    core::version::{Library, LibraryArtifact, LibraryDownloads},
    install::{
        assets::{asset_object_path, plan_asset_object_downloads_from_index, AssetIndexJson},
        client::{load_version_json, write_version_json},
        natives::extract_natives_for_platform,
        vanilla::plan_vanilla_downloads,
    },
    platform::{Arch, Os, Platform},
};
use zip::write::SimpleFileOptions;

#[test]
fn plans_asset_object_downloads_from_index() {
    let index: AssetIndexJson = serde_json::from_str(
        r#"{
            "objects": {
                "minecraft/sounds/example.ogg": {
                    "hash": "abcdef0123456789abcdef0123456789abcdef01",
                    "size": 12
                }
            }
        }"#,
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();

    let plan = plan_asset_object_downloads_from_index(&index, dir.path());

    assert_eq!(plan.tasks.len(), 1);
    assert_eq!(
        plan.tasks[0].destination,
        asset_object_path(dir.path(), "abcdef0123456789abcdef0123456789abcdef01")
    );
    assert!(plan.tasks[0]
        .url
        .ends_with("/ab/abcdef0123456789abcdef0123456789abcdef01"));
}

#[test]
fn extracts_native_artifacts_for_current_platform() {
    let dir = tempfile::tempdir().unwrap();
    let jar_path = dir
        .path()
        .join("libraries/org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1-natives-macos-arm64.jar");
    std::fs::create_dir_all(jar_path.parent().unwrap()).unwrap();
    let file = File::create(&jar_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("liblwjgl.dylib", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"native").unwrap();
    zip.start_file("META-INF/ignored", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"ignored").unwrap();
    zip.finish().unwrap();
    let library = Library {
        name: "org.lwjgl:lwjgl:3.3.1:natives-macos-arm64".to_string(),
        url: None,
        rules: Vec::new(),
        downloads: Some(LibraryDownloads {
            artifact: Some(LibraryArtifact {
                path: "org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1-natives-macos-arm64.jar".to_string(),
                url: "https://example.invalid/native.jar".to_string(),
                sha1: "sha1".to_string(),
                size: 6,
            }),
            classifiers: Default::default(),
        }),
        natives: None,
        extract: None,
    };

    let natives_dir = extract_natives_for_platform(
        &[library],
        dir.path(),
        "fabric-loader-0.19.2-1.20.1",
        Platform {
            os: Os::MacOs,
            arch: Arch::Aarch64,
        },
    )
    .unwrap();

    assert_eq!(
        std::fs::read(natives_dir.join("liblwjgl.dylib")).unwrap(),
        b"native"
    );
    assert!(!natives_dir.join("META-INF/ignored").exists());
}

#[test]
fn inherited_version_downloads_client_to_parent_jar_path() {
    let version = serde_json::from_str(
        r#"{
            "id":"fabric-loader-0.19.2-1.20.1",
            "jar":"1.20.1",
            "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
            "downloads":{
                "client":{
                    "sha1":"client-sha1",
                    "size":1,
                    "url":"https://example.invalid/client.jar"
                }
            }
        }"#,
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();

    let plan = plan_vanilla_downloads(&version, dir.path()).unwrap();

    assert!(plan
        .tasks
        .iter()
        .any(|task| task.destination.ends_with("versions/1.20.1/1.20.1.jar")));
    assert!(!plan.tasks.iter().any(|task| task
        .destination
        .ends_with("versions/fabric-loader-0.19.2-1.20.1/fabric-loader-0.19.2-1.20.1.jar")));
}

#[test]
fn load_version_json_merges_inherited_profile_from_versions_dir() {
    let parent = serde_json::from_str(
        r#"{
            "id":"1.20.1",
            "mainClass":"net.minecraft.client.main.Main",
            "arguments":{"game":["--username","${auth_player_name}"],"jvm":[]},
            "libraries":[{"name":"com.example:parent:1.0"}]
        }"#,
    )
    .unwrap();
    let child = serde_json::from_str(
        r#"{
            "id":"fabric-loader-0.19.2-1.20.1",
            "inheritsFrom":"1.20.1",
            "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
            "arguments":{"game":[],"jvm":[]},
            "libraries":[{"name":"net.fabricmc:fabric-loader:0.19.2"}]
        }"#,
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    write_version_json(dir.path(), &parent).unwrap();
    write_version_json(dir.path(), &child).unwrap();

    let merged = load_version_json(dir.path(), "fabric-loader-0.19.2-1.20.1").unwrap();

    assert_eq!(
        merged.main_class.as_deref(),
        Some("net.fabricmc.loader.impl.launch.knot.KnotClient")
    );
    assert_eq!(merged.jar.as_deref(), Some("1.20.1"));
    assert_eq!(merged.libraries.len(), 2);
}
