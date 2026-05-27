use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use crate::{
    io::hash::sha1_file,
    progress::{ProgressEvent, ProgressReporter, SkipReason},
    LauncherError, Result,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checksum {
    Sha1(String),
    Sha256(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadTask {
    pub url: String,
    pub destination: PathBuf,
    pub checksum: Option<Checksum>,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DownloadPlan {
    pub tasks: Vec<DownloadTask>,
}

pub fn should_skip_existing(task: &DownloadTask) -> Result<bool> {
    if !task.destination.is_file() {
        return Ok(false);
    }

    match &task.checksum {
        Some(Checksum::Sha1(expected)) => Ok(sha1_file(&task.destination)? == *expected),
        Some(Checksum::Sha256(_)) => Ok(false),
        None => Ok(true),
    }
}

pub fn execute_plan(plan: &DownloadPlan, reporter: &mut dyn ProgressReporter) -> Result<()> {
    let client = super::http::client()?;
    for task in &plan.tasks {
        if should_skip_existing(task)? {
            reporter.report(ProgressEvent::TaskSkipped {
                label: task.label.clone(),
                reason: if task.checksum.is_some() {
                    SkipReason::ChecksumMatched
                } else {
                    SkipReason::FileExistsWithoutChecksum
                },
            });
            continue;
        }

        reporter.report(ProgressEvent::TaskStarted {
            label: task.label.clone(),
            path: task.destination.clone(),
        });

        if let Some(parent) = task.destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut response = client.get(&task.url).send()?.error_for_status()?;
        let mut file = File::create(&task.destination)?;
        io::copy(&mut response, &mut file)?;

        if let Some(Checksum::Sha1(expected)) = &task.checksum {
            let actual = sha1_file(&task.destination)?;
            if actual != *expected {
                return Err(LauncherError::ChecksumMismatch {
                    path: task.destination.clone(),
                    expected: expected.clone(),
                    actual,
                });
            }
        }

        reporter.report(ProgressEvent::TaskFinished {
            label: task.label.clone(),
        });
    }
    Ok(())
}
