use std::fs;

use mc_launcher_core::{
    core::version::VersionJson,
    install::loader::{
        installer_command_args, loader_profile_path, write_loader_profile, InstallerInvocation,
    },
    loader::LoaderKind,
};

#[test]
fn writes_loader_profile_to_versions_directory() {
    let dir = tempfile::tempdir().unwrap();
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/fabric_profile_1_20_4.json")).unwrap();

    let path = write_loader_profile(dir.path(), &profile).unwrap();

    assert_eq!(
        path,
        dir.path()
            .join("versions")
            .join("fabric-loader-0.15.7-1.20.4")
            .join("fabric-loader-0.15.7-1.20.4.json")
    );
    assert!(path.is_file());

    let saved: VersionJson = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(saved.id.as_deref(), Some("fabric-loader-0.15.7-1.20.4"));
}

#[test]
fn computes_loader_profile_path() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        loader_profile_path(dir.path(), "quilt-loader-0.23.1-1.20.4"),
        dir.path()
            .join("versions")
            .join("quilt-loader-0.23.1-1.20.4")
            .join("quilt-loader-0.23.1-1.20.4.json")
    );
}

#[test]
fn builds_installer_command_arguments() {
    let invocation = InstallerInvocation {
        loader: LoaderKind::Forge,
        java_executable: "java".into(),
        installer_path: "/tmp/forge-installer.jar".into(),
        minecraft_dir: "/tmp/mc".into(),
    };

    assert_eq!(
        installer_command_args(&invocation),
        vec![
            "-jar".to_string(),
            "/tmp/forge-installer.jar".to_string(),
            "--installClient".to_string(),
            "/tmp/mc".to_string()
        ]
    );
}
