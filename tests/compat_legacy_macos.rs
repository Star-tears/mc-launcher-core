use std::path::PathBuf;

use mc_launcher_core::{
    account::Account,
    command::builder::{build_launch_command_for_platform, LaunchOptions},
    compatibility::{
        apply_compatibility, CompatibilityPatch, CompatibilityPolicy, WindowingStrategy,
    },
    core::{classpath::classpath_entries, version::VersionJson},
    install::vanilla::plan_vanilla_downloads_for_platform,
    platform::{Arch, Os, Platform},
};

fn mac_arm64() -> Platform {
    Platform {
        os: Os::MacOs,
        arch: Arch::Aarch64,
    }
}

fn mac_x64() -> Platform {
    Platform {
        os: Os::MacOs,
        arch: Arch::X86_64,
    }
}

fn legacy_lwjgl2_version() -> VersionJson {
    serde_json::from_str(
        r#"{
            "id":"1.12.2",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "minecraftArguments":"--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type}",
            "assets":"1.12",
            "downloads":{
                "client":{
                    "sha1":"client-sha1",
                    "size":1,
                    "url":"https://example.test/client.jar"
                }
            },
            "libraries":[
                {
                    "name":"com.example:keep:1.0",
                    "downloads":{
                        "artifact":{
                            "path":"com/example/keep/1.0/keep-1.0.jar",
                            "sha1":"keep-sha1",
                            "size":10,
                            "url":"https://example.test/keep.jar"
                        }
                    }
                },
                {"name":"org.lwjgl.lwjgl:lwjgl:2.9.2-nightly-20140822"},
                {"name":"org.lwjgl.lwjgl:lwjgl_util:2.9.2-nightly-20140822"},
                {
                    "name":"org.lwjgl.lwjgl:lwjgl-platform:2.9.2-nightly-20140822",
                    "natives":{"osx":"natives-osx"},
                    "downloads":{
                        "classifiers":{
                            "natives-osx":{
                                "path":"org/lwjgl/lwjgl/lwjgl-platform/2.9.2-nightly-20140822/lwjgl-platform-2.9.2-nightly-20140822-natives-osx.jar",
                                "sha1":"old-native-sha1",
                                "size":10,
                                "url":"https://example.test/old-native.jar"
                            }
                        }
                    }
                },
                {"name":"net.java.jinput:jinput:2.0.5"},
                {
                    "name":"net.java.jinput:jinput-platform:2.0.5",
                    "natives":{"osx":"natives-osx"},
                    "downloads":{
                        "classifiers":{
                            "natives-osx":{
                                "path":"net/java/jinput/jinput-platform/2.0.5/jinput-platform-2.0.5-natives-osx.jar",
                                "sha1":"old-jinput-native-sha1",
                                "size":10,
                                "url":"https://example.test/old-jinput-native.jar"
                            }
                        }
                    }
                },
                {"name":"net.java.jutils:jutils:1.0.0"},
                {"name":"ca.weblite:java-objc-bridge:1.0.0"}
            ]
        }"#,
    )
    .unwrap()
}

fn modern_lwjgl3_version() -> VersionJson {
    serde_json::from_str(
        r#"{
            "id":"1.18.2",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "arguments":{"game":[],"jvm":[]},
            "downloads":{
                "client":{
                    "sha1":"client-sha1",
                    "size":1,
                    "url":"https://example.test/client.jar"
                }
            },
            "libraries":[
                {
                    "name":"com.example:keep:1.0",
                    "downloads":{
                        "artifact":{
                            "path":"com/example/keep/1.0/keep-1.0.jar",
                            "sha1":"keep-sha1",
                            "size":10,
                            "url":"https://example.test/keep.jar"
                        }
                    }
                },
                {"name":"ca.weblite:java-objc-bridge:1.1"},
                {"name":"org.lwjgl:lwjgl:3.2.2"},
                {"name":"org.lwjgl:lwjgl-glfw:3.2.2"},
                {
                    "name":"org.lwjgl:lwjgl:3.2.2",
                    "natives":{"osx":"natives-macos"},
                    "downloads":{
                        "classifiers":{
                            "natives-macos":{
                                "path":"org/lwjgl/lwjgl/3.2.2/lwjgl-3.2.2-natives-macos.jar",
                                "sha1":"old-native-sha1",
                                "size":10,
                                "url":"https://example.test/old-native.jar"
                            }
                        }
                    }
                }
            ]
        }"#,
    )
    .unwrap()
}

#[test]
fn recommends_legacy_lwjgl2_patch_only_for_macos_arm64() {
    let version = legacy_lwjgl2_version();

    let patched = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Auto);
    assert_eq!(
        patched.applied_patches,
        vec![CompatibilityPatch::LegacyMacArm64Lwjgl2]
    );
    assert_eq!(patched.java_runtime.unwrap().major_version, 8);
    assert_eq!(patched.java_runtime.unwrap().arch, Arch::Aarch64);

    let x64 = apply_compatibility(&version, mac_x64(), CompatibilityPolicy::Auto);
    assert!(x64.applied_patches.is_empty());

    let disabled = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Disabled);
    assert!(disabled.applied_patches.is_empty());
}

#[test]
fn reports_app_host_windowing_strategy_for_legacy_lwjgl2_on_macos_arm64() {
    let version = legacy_lwjgl2_version();

    let patched = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Auto);

    assert_eq!(
        patched.windowing.strategy,
        WindowingStrategy::MacOsAppBundle
    );
    assert!(patched.windowing.requires_visible_window_verification);
    assert!(patched.windowing.reason.contains("LWJGL 2"));
}

#[test]
fn keeps_standard_windowing_strategy_when_legacy_patch_is_not_applied() {
    let version = legacy_lwjgl2_version();

    let x64 = apply_compatibility(&version, mac_x64(), CompatibilityPolicy::Auto);
    assert_eq!(x64.windowing.strategy, WindowingStrategy::CurrentProcess);
    assert!(!x64.windowing.requires_visible_window_verification);

    let disabled = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Disabled);
    assert_eq!(
        disabled.windowing.strategy,
        WindowingStrategy::CurrentProcess
    );
    assert!(!disabled.windowing.requires_visible_window_verification);
}

#[test]
fn replaces_legacy_lwjgl2_libraries_with_arm64_metadata() {
    let version = legacy_lwjgl2_version();
    let patched = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Auto);
    let names = patched
        .version
        .libraries
        .iter()
        .map(|library| library.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"com.example:keep:1.0"));
    assert!(!names
        .iter()
        .any(|name| name.starts_with("org.lwjgl.lwjgl:lwjgl:2.9.2")));
    assert!(!names
        .iter()
        .any(|name| name.starts_with("ca.weblite:java-objc-bridge:1.0.0")));
    assert!(names.contains(&"org.lwjgl.lwjgl:lwjgl:2.9.4-nightly-20150209"));
    assert!(names.contains(&"org.lwjgl.lwjgl:lwjgl-platform:2.9.4-nightly-20150209-mmachina.2"));
    assert!(names.contains(&"ca.weblite:java-objc-bridge:1.1.0-mmachina.1"));
}

#[test]
fn plans_patched_lwjgl2_artifacts_and_native_classifiers() {
    let version = legacy_lwjgl2_version();
    let dir = tempfile::tempdir().unwrap();

    let plan = plan_vanilla_downloads_for_platform(
        &version,
        dir.path(),
        mac_arm64(),
        CompatibilityPolicy::Auto,
    )
    .unwrap();
    let urls = plan
        .tasks
        .iter()
        .map(|task| task.url.as_str())
        .collect::<Vec<_>>();

    assert!(urls
        .iter()
        .any(|url| url
            .contains("MinecraftMachina/lwjgl/releases/download/2.9.4-20150209-mmachina.2")));
    assert!(urls.iter().any(|url| url
        .contains("MinecraftMachina/Java-Objective-C-Bridge/releases/download/1.1.0-mmachina.1")));
    assert!(plan.tasks.iter().any(|task| task
        .destination
        .to_string_lossy()
        .ends_with("lwjgl-platform-2.9.4-nightly-20150209-natives-osx.jar")));
}

#[test]
fn classpath_uses_download_artifact_paths_after_patch() {
    let version = legacy_lwjgl2_version();
    let patched = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Auto);

    let entries = classpath_entries(&patched.version, "/tmp/mc").unwrap();
    let classpath = entries
        .iter()
        .map(|entry| entry.to_string_lossy())
        .collect::<Vec<_>>()
        .join(":");

    assert!(classpath.contains(
        "org/lwjgl/lwjgl/lwjgl-platform/2.9.4-nightly-20150209/lwjgl-platform-2.9.4-nightly-20150209.jar"
    ));
    assert!(!classpath.contains(
        "2.9.4-nightly-20150209-mmachina.2/lwjgl-platform-2.9.4-nightly-20150209-mmachina.2.jar"
    ));
}

#[test]
fn replaces_lwjgl3_libraries_with_arm64_metadata_on_macos_arm64() {
    let version = modern_lwjgl3_version();

    let patched = apply_compatibility(&version, mac_arm64(), CompatibilityPolicy::Auto);
    let names = patched
        .version
        .libraries
        .iter()
        .map(|library| library.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        patched.applied_patches,
        vec![CompatibilityPatch::MacArm64Lwjgl3]
    );
    assert!(names.contains(&"com.example:keep:1.0"));
    assert!(!names
        .iter()
        .any(|name| name.starts_with("org.lwjgl:lwjgl:3.2.2")));
    assert!(names.contains(&"org.lwjgl:lwjgl:3.3.1-mmachina.1"));
    assert!(names.contains(&"org.lwjgl:lwjgl-glfw:3.3.1-mmachina.1"));
    assert!(names.contains(&"ca.weblite:java-objc-bridge:1.1.0-mmachina.1"));
}

#[test]
fn plans_lwjgl3_arm64_native_classifiers() {
    let version = modern_lwjgl3_version();
    let dir = tempfile::tempdir().unwrap();

    let plan = plan_vanilla_downloads_for_platform(
        &version,
        dir.path(),
        mac_arm64(),
        CompatibilityPolicy::Auto,
    )
    .unwrap();

    assert!(plan.tasks.iter().any(|task| task
        .url
        .contains("MinecraftMachina/lwjgl3/releases/download/3.3.1-mmachina.1")));
    assert!(plan.tasks.iter().any(|task| task
        .destination
        .to_string_lossy()
        .ends_with("lwjgl-3.3.1-mmachina.1-natives-macos.jar")));
}

#[test]
fn legacy_launch_command_gets_default_jvm_arguments() {
    let version = legacy_lwjgl2_version();

    let command = build_launch_command_for_platform(
        &version,
        PathBuf::from("/tmp/mc"),
        LaunchOptions {
            account: Account::offline("Steve"),
            java_executable: Some(PathBuf::from("java")),
            ..Default::default()
        },
        mac_arm64(),
    )
    .unwrap();

    assert!(command.args.contains(&"-XstartOnFirstThread".to_string()));
    assert!(command
        .args
        .iter()
        .any(|arg| arg.starts_with("-Djava.library.path=/tmp/mc/versions/1.12.2/natives")));
    assert!(command.args.contains(&"-cp".to_string()));
    assert!(command
        .args
        .contains(&"net.minecraft.client.main.Main".to_string()));
    assert!(command.args.contains(&"Steve".to_string()));
}
