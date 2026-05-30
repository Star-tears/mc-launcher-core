//! Error types returned by the launcher facade and lower-level modules.

use std::path::PathBuf;

use crate::loader::LoaderKind;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, LauncherError>;

/// Errors produced by install, metadata, IO, and launch-command operations.
#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    /// HTTP or request-building failure.
    #[error("network error: {source}")]
    Network {
        /// Original reqwest error.
        #[from]
        source: reqwest::Error,
    },
    /// Filesystem failure.
    #[error("io error: {source}")]
    Io {
        /// Original IO error.
        #[from]
        source: std::io::Error,
    },
    /// JSON parse or serialization failure.
    #[error("json error: {source}")]
    Json {
        /// Original serde_json error.
        #[from]
        source: serde_json::Error,
    },
    /// ZIP archive failure.
    #[error("zip error: {source}")]
    Zip {
        /// Original zip error.
        #[from]
        source: zip::result::ZipError,
    },
    /// Requested version id was not found or was not valid in context.
    #[error("invalid version id: {id}")]
    InvalidVersionId {
        /// Requested version id.
        id: String,
    },
    /// Current or requested platform cannot be handled.
    #[error("unsupported platform: {os}/{arch}")]
    UnsupportedPlatform {
        /// Operating system name.
        os: String,
        /// CPU architecture name.
        arch: String,
    },
    /// A downloaded file did not match its expected checksum.
    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// File that failed validation.
        path: PathBuf,
        /// Expected checksum.
        expected: String,
        /// Actual checksum.
        actual: String,
    },
    /// A joined path escaped the intended base directory.
    #[error("unsafe path {path} escapes base {base}")]
    UnsafePath {
        /// Expected base directory.
        base: PathBuf,
        /// Path that escaped the base directory.
        path: PathBuf,
    },
    /// Maven coordinate parsing failed.
    #[error("invalid maven coordinate: {coordinate}")]
    InvalidMavenCoordinate {
        /// Coordinate string that could not be parsed.
        coordinate: String,
    },
    /// A requested loader version could not be resolved.
    #[error("{loader:?} loader version not found: {version}")]
    LoaderVersionNotFound {
        /// Loader family being resolved.
        loader: LoaderKind,
        /// Requested loader version selector or value.
        version: String,
    },
    /// A Forge or NeoForge installer process failed.
    #[error("{loader:?} installer failed with status {status:?}")]
    InstallerFailed {
        /// Loader family whose installer failed.
        loader: LoaderKind,
        /// Process exit status code, if available.
        status: Option<i32>,
    },
    /// Required metadata was missing from a version/profile document.
    #[error("missing field {field} in {context}")]
    MissingField {
        /// Metadata document or profile being read.
        context: String,
        /// Missing field name.
        field: String,
    },
    /// Miscellaneous error with a caller-facing message.
    #[error("{message}")]
    Other {
        /// Error message.
        message: String,
    },
}
