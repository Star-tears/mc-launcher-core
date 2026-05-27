use mc_launcher_core::loader::{
    forge::{forge_installed_version_id, parse_maven_metadata as parse_forge_metadata},
    neoforge::{neoforge_installed_version_id, parse_maven_metadata as parse_neoforge_metadata},
};

#[test]
fn parses_forge_maven_metadata() {
    let metadata = parse_forge_metadata(include_str!("fixtures/forge_maven_metadata.xml")).unwrap();
    assert_eq!(metadata.latest, "1.21.4-54.1.6");
    assert_eq!(metadata.versions, vec!["1.20.4-49.0.50", "1.21.4-54.1.6"]);
}

#[test]
fn maps_forge_version_to_installed_id() {
    assert_eq!(
        forge_installed_version_id("1.20.4-49.0.50").unwrap(),
        "1.20.4-forge-49.0.50"
    );
}

#[test]
fn parses_neoforge_maven_metadata() {
    let metadata =
        parse_neoforge_metadata(include_str!("fixtures/neoforge_maven_metadata.xml")).unwrap();
    assert_eq!(metadata.latest, "21.4.150");
    assert_eq!(metadata.versions, vec!["20.4.240", "21.4.150"]);
}

#[test]
fn maps_neoforge_version_to_installed_id() {
    assert_eq!(
        neoforge_installed_version_id("1.21.4", "21.4.150"),
        "neoforge-21.4.150"
    );
}
