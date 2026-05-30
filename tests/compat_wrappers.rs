#![allow(deprecated)]

use mc_launcher_core::{
    auth::offline::get_offline_options, command::get_minecraft_command,
    install::install_minecraft_version, utils::get_core_version,
};

#[test]
fn offline_options_still_returns_username() {
    let options = get_offline_options("Steve");
    assert_eq!(options.username.as_deref(), Some("Steve"));
    assert!(options.uuid.is_some());
}

#[test]
fn core_version_is_package_version() {
    assert_eq!(get_core_version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn missing_version_command_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = get_minecraft_command("missing", dir.path(), &Default::default()).unwrap_err();
    assert!(err.to_string().contains("missing"));
}

#[test]
#[ignore = "performs a complete network install"]
fn vanilla_install_wrapper_accepts_request_shape() {
    let dir = tempfile::tempdir().unwrap();
    let result = install_minecraft_version("1.20.4", dir.path(), &Default::default());
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("network"));
}
