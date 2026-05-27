use mc_launcher_core::loader::{fabric, forge, neoforge, quilt};

#[test]
#[ignore = "requires network"]
fn fetches_live_fabric_loader_versions() {
    assert!(!fabric::list_loader_versions().unwrap().is_empty());
}

#[test]
#[ignore = "requires network"]
fn fetches_live_quilt_loader_versions() {
    assert!(!quilt::list_loader_versions().unwrap().is_empty());
}

#[test]
#[ignore = "requires network"]
fn fetches_live_forge_versions() {
    assert!(!forge::list_forge_versions().unwrap().is_empty());
}

#[test]
#[ignore = "requires network"]
fn fetches_live_neoforge_versions() {
    assert!(!neoforge::list_neoforge_versions().unwrap().is_empty());
}
