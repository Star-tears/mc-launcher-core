use crate::loader::LoaderKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderVersion {
    Latest,
    LatestStable,
    Exact(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderSpec {
    Fabric { version: LoaderVersion },
    Quilt { version: LoaderVersion },
    Forge { version: LoaderVersion },
    NeoForge { version: LoaderVersion },
}

impl LoaderSpec {
    pub fn kind(&self) -> LoaderKind {
        match self {
            Self::Fabric { .. } => LoaderKind::Fabric,
            Self::Quilt { .. } => LoaderKind::Quilt,
            Self::Forge { .. } => LoaderKind::Forge,
            Self::NeoForge { .. } => LoaderKind::NeoForge,
        }
    }
}
