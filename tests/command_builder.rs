use std::path::PathBuf;

use mc_launcher_core::{
    account::Account,
    command::builder::{build_launch_command, LaunchCommand, LaunchOptions},
    core::version::VersionJson,
};

#[test]
fn builds_basic_modern_launch_command() {
    let version: VersionJson = serde_json::from_str(
        r#"{
            "id":"1.20.4",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "minimumLauncherVersion":21,
            "arguments":{
                "jvm":["-Djava.library.path=${natives_directory}","-cp","${classpath}"],
                "game":["--username","${auth_player_name}","--version","${version_name}","--gameDir","${game_directory}","--assetsDir","${assets_root}","--assetIndex","${assets_index_name}","--uuid","${auth_uuid}","--accessToken","${auth_access_token}","--userType","${user_type}","--versionType","${version_type}"]
            },
            "assets":"12",
            "libraries":[{"name":"com.example:demo:1.0"}]
        }"#,
    )
    .unwrap();

    let command = build_launch_command(
        &version,
        PathBuf::from("/tmp/mc"),
        LaunchOptions {
            account: Account::offline("Steve"),
            java_executable: Some(PathBuf::from("/usr/bin/java")),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(command.executable, PathBuf::from("/usr/bin/java"));
    assert!(command
        .args
        .contains(&"net.minecraft.client.main.Main".to_string()));
    assert!(command.args.contains(&"Steve".to_string()));
    assert!(command
        .args
        .iter()
        .any(|arg| arg.contains("libraries/com/example/demo/1.0/demo-1.0.jar")));
    assert_eq!(
        command.working_dir,
        PathBuf::from("/tmp/mc/versions/1.20.4")
    );
    assert!(command
        .args
        .windows(2)
        .any(|window| window == ["--gameDir", "/tmp/mc/versions/1.20.4"]));
}

#[test]
fn launch_command_exposes_process_parts() {
    let command = LaunchCommand {
        executable: PathBuf::from("java"),
        args: vec!["-version".to_string()],
        working_dir: PathBuf::from("/tmp/mc"),
        env: Vec::new(),
    };

    assert_eq!(command.to_process_parts().0, PathBuf::from("java"));
    assert_eq!(command.to_process_parts().1, vec!["-version".to_string()]);
}

#[test]
fn explicit_game_directory_overrides_default_version_isolation() {
    let version: VersionJson = serde_json::from_str(
        r#"{
            "id":"1.20.4",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "arguments":{
                "jvm":["-cp","${classpath}"],
                "game":["--gameDir","${game_directory}"]
            }
        }"#,
    )
    .unwrap();

    let command = build_launch_command(
        &version,
        PathBuf::from("/tmp/mc"),
        LaunchOptions {
            game_directory: Some(PathBuf::from("/tmp/custom-instance")),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(command.working_dir, PathBuf::from("/tmp/custom-instance"));
    assert!(command
        .args
        .windows(2)
        .any(|window| window == ["--gameDir", "/tmp/custom-instance"]));
}
