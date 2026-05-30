//! Common imports for launcher applications.
//!
//! This module re-exports the facade types most callers need for installation,
//! launch command construction, progress reporting, compatibility handling, and
//! error handling.

pub use crate::{
    account::Account,
    command::builder::{LaunchCommand, LaunchOptions},
    compatibility::{
        CompatibilityPatch, CompatibilityPolicy, CompatibilityResult, JavaRuntimeHint,
        WindowingHint, WindowingStrategy,
    },
    error::{LauncherError, Result},
    install::request::{InstallRequest, InstallResult, JavaInstallPolicy},
    launcher::Launcher,
    loader::{
        common::{LoaderSpec, LoaderVersion},
        LoaderKind,
    },
    progress::{ProgressEvent, ProgressReporter},
};
