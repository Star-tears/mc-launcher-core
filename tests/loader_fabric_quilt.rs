use mc_launcher_core::{
    core::version::VersionJson,
    loader::{
        fabric::{latest_stable_loader as latest_fabric_stable, FabricLoaderVersion},
        quilt::{latest_loader as latest_quilt_loader, QuiltLoaderVersion},
    },
};

#[test]
fn fabric_latest_stable_prefers_stable_flag() {
    let versions: Vec<FabricLoaderVersion> =
        serde_json::from_str(include_str!("fixtures/fabric_loaders.json")).unwrap();
    assert_eq!(latest_fabric_stable(&versions).unwrap().version, "0.19.2");
}

#[test]
fn quilt_latest_loader_uses_first_entry() {
    let versions: Vec<QuiltLoaderVersion> =
        serde_json::from_str(include_str!("fixtures/quilt_loaders.json")).unwrap();
    assert_eq!(
        latest_quilt_loader(&versions).unwrap().version,
        "0.30.0-beta.7"
    );
}

#[test]
fn fabric_profile_parses_as_version_json() {
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/fabric_profile_1_20_4.json")).unwrap();
    assert_eq!(profile.id.as_deref(), Some("fabric-loader-0.15.7-1.20.4"));
    assert_eq!(profile.inherits_from.as_deref(), Some("1.20.4"));
}

#[test]
fn quilt_profile_parses_as_version_json() {
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/quilt_profile_1_20_4.json")).unwrap();
    assert_eq!(profile.id.as_deref(), Some("quilt-loader-0.23.1-1.20.4"));
    assert_eq!(profile.inherits_from.as_deref(), Some("1.20.4"));
}
