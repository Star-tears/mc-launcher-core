//! Hashing helpers for installed files.

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use sha1::{Digest, Sha1};

use crate::Result;

/// Calculates the SHA-1 digest of a file as lowercase hexadecimal.
///
/// # Errors
///
/// Returns [`crate::LauncherError`] if the file cannot be read.
pub fn sha1_file(path: impl AsRef<Path>) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
