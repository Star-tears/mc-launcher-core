use mc_launcher_core::loader::{
    forge::{
        forge_installed_version_id, latest_for_minecraft as latest_forge_for_minecraft,
        parse_maven_metadata as parse_forge_metadata,
    },
    neoforge::{
        latest_for_minecraft as latest_neoforge_for_minecraft, neoforge_installed_version_id,
        parse_maven_metadata as parse_neoforge_metadata,
    },
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
fn selects_latest_forge_version_for_requested_minecraft_version() {
    let versions = vec![
        "1.20.4-49.0.50".to_string(),
        "1.21.4-54.1.6".to_string(),
        "1.20.4-49.2.8".to_string(),
    ];

    assert_eq!(
        latest_forge_for_minecraft(&versions, "1.20.4").unwrap(),
        "1.20.4-49.2.8"
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

#[test]
fn selects_latest_neoforge_version_for_requested_minecraft_version() {
    let versions = vec![
        "21.4.150".to_string(),
        "21.5.96".to_string(),
        "21.4.157".to_string(),
    ];

    assert_eq!(
        latest_neoforge_for_minecraft(&versions, "1.21.4").unwrap(),
        "21.4.157"
    );
}

#[test]
fn maps_minecraft_release_without_patch_to_neoforge_zero_patch_line() {
    let versions = vec!["21.0.167".to_string(), "21.1.244".to_string()];

    assert_eq!(
        latest_neoforge_for_minecraft(&versions, "1.21").unwrap(),
        "21.0.167"
    );
}

#[test]
fn selects_latest_neoforge_for_calendar_versioned_minecraft() {
    let versions = vec![
        "26.1.2.86".to_string(),
        "26.1.3.1".to_string(),
        "26.1.2.87".to_string(),
    ];

    assert_eq!(
        latest_neoforge_for_minecraft(&versions, "26.1.2").unwrap(),
        "26.1.2.87"
    );
}

#[test]
fn maps_calendar_release_without_patch_to_zero_patch_line() {
    let versions = vec!["26.1.0.42".to_string(), "26.1.2.87".to_string()];

    assert_eq!(
        latest_neoforge_for_minecraft(&versions, "26.1").unwrap(),
        "26.1.0.42"
    );
}
