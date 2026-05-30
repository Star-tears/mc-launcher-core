//! Progress events emitted while installation work is performed.
//!
//! Callers can pass a closure to
//! [`crate::launcher::Launcher::install_with_progress`] because closures that
//! accept [`ProgressEvent`] automatically implement [`ProgressReporter`].

use std::path::PathBuf;

/// Coarse install stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStage {
    /// Resolve version metadata.
    ResolveVersion,
    /// Download libraries.
    DownloadLibraries,
    /// Download asset index and objects.
    DownloadAssets,
    /// Install a Java runtime.
    InstallRuntime,
    /// Extract native libraries.
    ExtractNatives,
    /// Run or write loader installation metadata.
    LoaderInstall,
    /// Verify installed files.
    Verify,
}

/// Reason a download task was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Existing file matched the expected checksum.
    ChecksumMatched,
    /// Existing file was accepted because no checksum was available.
    FileExistsWithoutChecksum,
}

/// Event emitted during install and download operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    /// A coarse install stage started.
    StageStarted {
        /// Stage that started.
        stage: InstallStage,
    },
    /// A concrete file task started.
    TaskStarted {
        /// Human-readable task label.
        label: String,
        /// Destination path being written.
        path: PathBuf,
    },
    /// A concrete file task was skipped.
    TaskSkipped {
        /// Human-readable task label.
        label: String,
        /// Why the task did not need to run.
        reason: SkipReason,
    },
    /// A concrete file task finished.
    TaskFinished {
        /// Human-readable task label.
        label: String,
    },
    /// Bytes were received for a task.
    BytesReceived {
        /// Human-readable task label.
        label: String,
        /// Bytes received so far.
        received: u64,
        /// Total byte count when the server reported it.
        total: Option<u64>,
    },
}

/// Receives installation progress events.
pub trait ProgressReporter {
    /// Handles a progress event.
    fn report(&mut self, event: ProgressEvent);
}

impl<F> ProgressReporter for F
where
    F: FnMut(ProgressEvent),
{
    fn report(&mut self, event: ProgressEvent) {
        self(event);
    }
}
