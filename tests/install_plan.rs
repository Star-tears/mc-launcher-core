use mc_launcher_core::{
    core::version::VersionJson,
    install::{
        libraries::plan_library_downloads_for_platform,
        request::{InstallRequest, JavaInstallPolicy},
        vanilla::plan_vanilla_downloads,
    },
    platform::Platform,
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

#[test]
fn plans_loader_libraries_without_download_metadata() {
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/fabric_profile_1_20_4.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let tasks =
        plan_library_downloads_for_platform(&profile.libraries, dir.path(), Platform::current())
            .unwrap();
    let urls = tasks
        .iter()
        .map(|task| task.url.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        urls,
        vec![
            "https://maven.fabricmc.net/net/fabricmc/intermediary/1.20.4/intermediary-1.20.4.jar",
            "https://maven.fabricmc.net/net/fabricmc/fabric-loader/0.15.7/fabric-loader-0.15.7.jar",
        ]
    );
}
