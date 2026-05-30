//! Archive extraction helpers.

use std::{
    fs::{self, File},
    io,
    path::Path,
};

use zip::ZipArchive;

use crate::{io::paths::safe_join, Result};

/// Extracts a ZIP archive while rejecting entries that escape the destination.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the archive cannot be read, a file cannot
/// be written, or an entry path is unsafe.
pub fn extract_zip_safely(zip_path: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let destination = destination.as_ref();
    fs::create_dir_all(destination)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed) = entry.enclosed_name().map(|path| path.to_path_buf()) else {
            let unsafe_name = entry.name().to_string();
            safe_join(destination, unsafe_name)?;
            continue;
        };
        let output = safe_join(destination, enclosed)?;

        if entry.is_dir() {
            fs::create_dir_all(&output)?;
            continue;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = File::create(output)?;
        io::copy(&mut entry, &mut output_file)?;
    }

    Ok(())
}
