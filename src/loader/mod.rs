//! Loader metadata and installer helpers.
//!
//! Fabric and Quilt profiles are fetched and written directly from their
//! metadata APIs. Forge and NeoForge expose metadata helpers plus installer URLs
//! used by the high-level [`crate::launcher::Launcher`] facade.

pub mod common;
pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod quilt;

/// Supported mod loader families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    /// Fabric loader.
    Fabric,
    /// Quilt loader.
    Quilt,
    /// Forge loader.
    Forge,
    /// NeoForge loader.
    NeoForge,
}
