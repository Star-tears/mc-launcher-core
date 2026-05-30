//! Shared loader selection types.

use crate::loader::LoaderKind;

/// How to choose a mod loader version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderVersion {
    /// Use the newest loader version advertised by the loader metadata source.
    Latest,
    /// Use the newest stable loader version when the metadata source exposes
    /// stability information.
    LatestStable,
    /// Use an exact loader version string.
    Exact(String),
}

/// Loader profile requested during installation.
///
/// The Minecraft version lives on [`crate::install::request::InstallRequest`];
/// this enum only identifies the loader family and loader-version selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderSpec {
    /// Install a Fabric profile.
    Fabric { version: LoaderVersion },
    /// Install a Quilt profile.
    Quilt { version: LoaderVersion },
    /// Install a Forge profile.
    Forge { version: LoaderVersion },
    /// Install a NeoForge profile.
    NeoForge { version: LoaderVersion },
}

impl LoaderSpec {
    /// Returns the loader family represented by this spec.
    pub fn kind(&self) -> LoaderKind {
        match self {
            Self::Fabric { .. } => LoaderKind::Fabric,
            Self::Quilt { .. } => LoaderKind::Quilt,
            Self::Forge { .. } => LoaderKind::Forge,
            Self::NeoForge { .. } => LoaderKind::NeoForge,
        }
    }
}
