use mc_launcher_core::{
    core::version::VersionJson,
    install::{
        request::{InstallRequest, JavaInstallPolicy},
        vanilla::plan_vanilla_downloads,
    },
};

#[test]
fn plans_client_library_and_asset_index_downloads() {
    let version: VersionJson =
        serde_json::from_str(include_str!("fixtures/version_1_20_4_min.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let plan = plan_vanilla_downloads(&version, dir.path()).unwrap();

    let destinations = plan
        .tasks
        .iter()
        .map(|task| task.destination.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    assert!(destinations
        .iter()
        .any(|path| path.ends_with("versions/1.20.4/1.20.4.jar")));
    assert!(destinations
        .iter()
        .any(|path| path.ends_with("libraries/com/example/demo/1.0/demo-1.0.jar")));
    assert!(destinations
        .iter()
        .any(|path| path.ends_with("assets/indexes/12.json")));
}

#[test]
fn install_request_defaults_to_auto_java() {
    let request = InstallRequest::vanilla("1.20.4");
    assert_eq!(request.minecraft_version, "1.20.4");
    assert_eq!(request.java, JavaInstallPolicy::Auto);
}
