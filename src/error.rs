use std::path::PathBuf;

use crate::loader::LoaderKind;

pub type Result<T> = std::result::Result<T, LauncherError>;

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("network error: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },
    #[error("io error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("json error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
    #[error("invalid version id: {id}")]
    InvalidVersionId { id: String },
    #[error("unsupported platform: {os}/{arch}")]
    UnsupportedPlatform { os: String, arch: String },
    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    #[error("unsafe path {path} escapes base {base}")]
    UnsafePath { base: PathBuf, path: PathBuf },
    #[error("invalid maven coordinate: {coordinate}")]
    InvalidMavenCoordinate { coordinate: String },
    #[error("{loader:?} loader version not found: {version}")]
    LoaderVersionNotFound { loader: LoaderKind, version: String },
    #[error("{loader:?} installer failed with status {status:?}")]
    InstallerFailed {
        loader: LoaderKind,
        status: Option<i32>,
    },
    #[error("missing field {field} in {context}")]
    MissingField { context: String, field: String },
    #[error("{message}")]
    Other { message: String },
}
