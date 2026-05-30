//! Install request and result types used by [`crate::launcher::Launcher`].

use crate::loader::common::LoaderSpec;

/// Describes the profile that should be installed.
///
/// A request always starts from a Minecraft version. Setting [`loader`] asks the
/// installer to create or run the corresponding loader profile for that
/// Minecraft version.
///
/// [`loader`]: InstallRequest::loader
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    /// Vanilla Minecraft version, such as `1.20.1`.
    pub minecraft_version: String,
    /// Optional loader profile to install on top of the Minecraft version.
    pub loader: Option<LoaderSpec>,
    /// Java runtime policy for installers that need to execute Java.
    pub java: JavaInstallPolicy,
}

impl InstallRequest {
    /// Creates a vanilla install request for the given Minecraft version.
    pub fn vanilla(version: impl Into<String>) -> Self {
        Self {
            minecraft_version: version.into(),
            loader: None,
            java: JavaInstallPolicy::Auto,
        }
    }
}

/// Controls how install code should handle Java runtime needs.
///
/// The current high-level facade does not bundle Java. `Auto` is retained as
/// the default policy for future runtime management and compatibility with the
/// public request shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaInstallPolicy {
    /// Allow the launcher core to manage Java if a future implementation can do so.
    Auto,
    /// Never install or manage Java automatically.
    Never,
}

/// Result returned after an install completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    /// Version/profile id that should be loaded and launched.
    pub version_id: String,
}
