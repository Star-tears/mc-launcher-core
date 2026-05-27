use mc_launcher_core::{LauncherError, Result};

fn returns_error() -> Result<()> {
    Err(LauncherError::InvalidVersionId {
        id: "bad/version".to_string(),
    })
}

#[test]
fn launcher_error_formats_context() {
    let err = returns_error().unwrap_err();
    assert!(err.to_string().contains("bad/version"));
}
