use std::{fs, path::PathBuf};

use mc_launcher_core::command::{
    builder::LaunchCommand,
    macos_app::{prepare_macos_app_bundle, MacOsAppBundleOptions},
};
use mc_launcher_core::launcher::Launcher;

#[test]
fn writes_macos_app_bundle_and_returns_open_command() {
    let dir = tempfile::tempdir().unwrap();
    let bundle_path = dir.path().join("Minecraft Legacy Host.app");
    let launch = LaunchCommand {
        executable: PathBuf::from(
            "/Library/Java/JavaVirtualMachines/zulu-8.jdk/Contents/Home/bin/java",
        ),
        args: vec![
            "-XstartOnFirstThread".to_string(),
            "-Djava.library.path=/tmp/mc/versions/1.12.2/natives".to_string(),
            "-cp".to_string(),
            "/tmp/mc/libraries/a.jar:/tmp/mc/libraries/b.jar".to_string(),
            "net.minecraft.client.main.Main".to_string(),
            "--username".to_string(),
            "Steve's Friend".to_string(),
        ],
        working_dir: PathBuf::from("/tmp/mc"),
        env: vec![("JAVA_TOOL_OPTIONS".to_string(), "-Dfoo=bar".to_string())],
    };

    let hosted = prepare_macos_app_bundle(
        &launch,
        &bundle_path,
        MacOsAppBundleOptions {
            bundle_name: "Minecraft Legacy Host".to_string(),
            bundle_identifier: "dev.mc-launcher-core.legacy-host".to_string(),
            stdout_path: Some(dir.path().join("minecraft.stdout.log")),
            stderr_path: Some(dir.path().join("minecraft.stderr.log")),
        },
    )
    .unwrap();

    assert_eq!(hosted.bundle_path, bundle_path);
    assert_eq!(
        hosted.open_command.executable,
        PathBuf::from("/usr/bin/open")
    );
    assert_eq!(
        hosted.open_command.args,
        vec![
            "-W".to_string(),
            "-n".to_string(),
            "--stdout".to_string(),
            dir.path()
                .join("minecraft.stdout.log")
                .to_string_lossy()
                .to_string(),
            "--stderr".to_string(),
            dir.path()
                .join("minecraft.stderr.log")
                .to_string_lossy()
                .to_string(),
            hosted.bundle_path.to_string_lossy().to_string()
        ]
    );

    let info = fs::read_to_string(hosted.bundle_path.join("Contents/Info.plist")).unwrap();
    assert!(info.contains("<key>CFBundleExecutable</key>"));
    assert!(info.contains("<string>launch</string>"));
    assert!(info.contains("<string>dev.mc-launcher-core.legacy-host</string>"));

    let script = fs::read_to_string(hosted.bundle_path.join("Contents/MacOS/launch")).unwrap();
    assert!(script.starts_with("#!/bin/sh\n"));
    assert!(script.contains("cd '/tmp/mc'\n"));
    assert!(script.contains("exec env 'JAVA_TOOL_OPTIONS=-Dfoo=bar'"));
    assert!(
        script.contains("'/Library/Java/JavaVirtualMachines/zulu-8.jdk/Contents/Home/bin/java'")
    );
    assert!(script.contains("'Steve'\\''s Friend'"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(hosted.bundle_path.join("Contents/MacOS/launch"))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0);
    }
}

#[test]
fn launcher_prepares_macos_app_bundle_under_minecraft_directory() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = Launcher::new(dir.path());
    let command = LaunchCommand {
        executable: PathBuf::from("java"),
        args: vec!["-version".to_string()],
        working_dir: dir.path().to_path_buf(),
        env: Vec::new(),
    };

    let hosted = launcher
        .prepare_macos_app_bundle_launch(
            &command,
            MacOsAppBundleOptions {
                bundle_name: "Minecraft/Legacy:Host".to_string(),
                bundle_identifier: "dev.mc-launcher-core.legacy-host".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(
        hosted.bundle_path,
        dir.path()
            .join("launcher-hosts")
            .join("macos")
            .join("Minecraft_Legacy_Host.app")
    );
    assert!(hosted.bundle_path.join("Contents/MacOS/launch").is_file());
}
