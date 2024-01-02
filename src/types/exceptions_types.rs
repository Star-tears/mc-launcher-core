use std::{fmt, error::Error};

#[derive(Debug)]
pub struct InvalidChecksum {
    pub url: String,
    pub path: String,
    pub expected: String,
    pub actual: String,
}


impl fmt::Display for InvalidChecksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Checksum validation failed for URL: {}, Path: {}. Expected: {}, Actual: {}",
            self.url, self.path, self.expected, self.actual
        )
    }
}

impl Error for InvalidChecksum {}