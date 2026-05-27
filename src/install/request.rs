use crate::loader::common::LoaderSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    pub minecraft_version: String,
    pub loader: Option<LoaderSpec>,
    pub java: JavaInstallPolicy,
}

impl InstallRequest {
    pub fn vanilla(version: impl Into<String>) -> Self {
        Self {
            minecraft_version: version.into(),
            loader: None,
            java: JavaInstallPolicy::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaInstallPolicy {
    Auto,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    pub version_id: String,
}
