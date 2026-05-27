use std::path::{Component, Path, PathBuf};

use crate::{LauncherError, Result};

pub fn safe_join(base: impl AsRef<Path>, relative: impl AsRef<Path>) -> Result<PathBuf> {
    let base = base.as_ref();
    let relative = relative.as_ref();

    if relative.is_absolute() {
        return Err(LauncherError::UnsafePath {
            base: base.to_path_buf(),
            path: relative.to_path_buf(),
        });
    }

    for component in relative.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(LauncherError::UnsafePath {
                    base: base.to_path_buf(),
                    path: relative.to_path_buf(),
                });
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }

    Ok(base.join(relative))
}

pub fn ensure_inside_base(base: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()> {
    let base = base.as_ref();
    let path = path.as_ref();
    if !path.starts_with(base) {
        return Err(LauncherError::UnsafePath {
            base: base.to_path_buf(),
            path: path.to_path_buf(),
        });
    }
    Ok(())
}
