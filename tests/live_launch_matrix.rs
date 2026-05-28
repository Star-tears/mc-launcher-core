use std::{collections::HashSet, path::PathBuf};

use mc_launcher_core::{
    account::Account,
    command::builder::{build_launch_command, LaunchOptions},
    core::version::VersionJson,
    net::http::get_json,
};
use serde::Deserialize;

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

const CLASSIC_AND_MAINSTREAM: &[&str] = &[
    "1.7.10", "1.8.9", "1.12.2", "1.16.5", "1.18.2", "1.19.2", "1.20.1", "1.20.2",
];

#[derive(Debug, Deserialize)]
struct VersionManifest {
    latest: LatestVersions,
    versions: Vec<ManifestVersion>,
}

#[derive(Debug, Deserialize)]
struct LatestVersions {
    release: String,
}

#[derive(Debug, Deserialize)]
struct ManifestVersion {
    id: String,
    url: String,
}

#[test]
#[ignore = "requires network"]
fn builds_launch_commands_for_classic_mainstream_and_latest() {
    let manifest: VersionManifest = get_json(VERSION_MANIFEST_URL).unwrap();
    let mut seen = HashSet::new();
    let mut version_ids = CLASSIC_AND_MAINSTREAM
        .iter()
        .map(|version| (*version).to_string())
        .collect::<Vec<_>>();
    version_ids.push(manifest.latest.release.clone());
    version_ids.retain(|version| seen.insert(version.clone()));

    for version_id in version_ids {
        let entry = manifest
            .versions
            .iter()
            .find(|entry| entry.id == version_id)
            .unwrap_or_else(|| panic!("missing version {version_id} in Mojang manifest"));
        let version: VersionJson = get_json(&entry.url).unwrap();
        let main_class = version
            .main_class
            .clone()
            .unwrap_or_else(|| panic!("missing mainClass for {version_id}"));

        let command = build_launch_command(
            &version,
            PathBuf::from("/tmp/mc-launcher-core-live-matrix"),
            LaunchOptions {
                account: Account::offline("Steve"),
                java_executable: Some(PathBuf::from("java")),
                ..Default::default()
            },
        )
        .unwrap_or_else(|err| panic!("failed to build launch command for {version_id}: {err}"));

        assert_eq!(command.executable, PathBuf::from("java"));
        assert!(
            command.args.contains(&main_class),
            "{version_id} command does not contain main class {main_class}"
        );
        assert!(
            command.args.iter().any(|arg| arg == "Steve"),
            "{version_id} command does not contain offline username"
        );
        assert!(
            command.args.iter().all(|arg| !arg.contains("${")),
            "{version_id} command contains unresolved placeholders: {:?}",
            command
                .args
                .iter()
                .filter(|arg| arg.contains("${"))
                .collect::<Vec<_>>()
        );
    }
}
