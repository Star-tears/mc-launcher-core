use std::path::PathBuf;

use crate::{LauncherError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenCoordinate {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: String,
}

impl MavenCoordinate {
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
