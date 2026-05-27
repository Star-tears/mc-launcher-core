use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStage {
    ResolveVersion,
    DownloadLibraries,
    DownloadAssets,
    InstallRuntime,
    ExtractNatives,
    LoaderInstall,
    Verify,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    ChecksumMatched,
    FileExistsWithoutChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    StageStarted {
        stage: InstallStage,
    },
    TaskStarted {
        label: String,
        path: PathBuf,
    },
    TaskSkipped {
        label: String,
        reason: SkipReason,
    },
    TaskFinished {
        label: String,
    },
    BytesReceived {
        label: String,
        received: u64,
        total: Option<u64>,
    },
}

pub trait ProgressReporter {
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
