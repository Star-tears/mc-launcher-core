//! Maven coordinate parsing and local artifact paths.

use std::path::PathBuf;

use crate::{LauncherError, Result};

/// Parsed Maven coordinate.
///
/// Minecraft library metadata commonly uses `group:artifact:version`,
/// `group:artifact:version:classifier`, or the same values followed by
/// `@extension`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenCoordinate {
    /// Maven group id.
    pub group: String,
    /// Maven artifact id.
    pub artifact: String,
    /// Artifact version.
    pub version: String,
    /// Optional classifier.
    pub classifier: Option<String>,
    /// Artifact extension, usually `jar`.
    pub extension: String,
}

impl MavenCoordinate {
    /// Parses a Minecraft/Maven coordinate string.
    ///
    /// # Errors
    ///
    /// Returns [`crate::LauncherError`] if the coordinate is empty or does not
    /// contain the expected components.
    pub fn parse(input: &str) -> Result<Self> {
        let (without_ext, extension) = match input.split_once('@') {
            Some((left, ext)) if !ext.is_empty() => (left, ext),
            Some(_) => {
                return Err(LauncherError::InvalidMavenCoordinate {
                    coordinate: input.to_string(),
                });
            }
            None => (input, "jar"),
        };

        let parts: Vec<&str> = without_ext.split(':').collect();
        let (group, artifact, version, classifier) = match parts.as_slice() {
            [group, artifact, version]
                if !group.is_empty() && !artifact.is_empty() && !version.is_empty() =>
            {
                (*group, *artifact, *version, None)
            }
            [group, artifact, version, classifier]
                if !group.is_empty()
                    && !artifact.is_empty()
                    && !version.is_empty()
                    && !classifier.is_empty() =>
            {
                (*group, *artifact, *version, Some((*classifier).to_string()))
            }
            _ => {
                return Err(LauncherError::InvalidMavenCoordinate {
                    coordinate: input.to_string(),
                });
            }
        };

        Ok(Self {
            group: group.to_string(),
            artifact: artifact.to_string(),
            version: version.to_string(),
            classifier,
            extension: extension.to_string(),
        })
    }

    /// Returns the relative Maven repository path for this coordinate.
    pub fn artifact_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        for segment in self.group.split('.') {
            path.push(segment);
        }
        path.push(&self.artifact);
        path.push(&self.version);

        let classifier = self
            .classifier
            .as_ref()
            .map(|value| format!("-{value}"))
            .unwrap_or_default();
        path.push(format!(
            "{}-{}{}.{}",
            self.artifact, self.version, classifier, self.extension
        ));
        path
    }
}
