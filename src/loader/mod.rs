pub mod common;
pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod quilt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}
