# Launcher Core Redesign Phase 3: Loader Providers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Fabric, Quilt, Forge, and NeoForge metadata/profile provider support and loader profile installation.

**Architecture:** This phase implements metadata-profile loaders first, then installer-backed loader metadata, then writes loader profiles through install helpers. It deliberately keeps MRPack out of scope.

**Tech Stack:** Rust 2021, `reqwest` blocking client, `serde`, `serde_json`, `thiserror`, `once_cell`, RustCrypto `sha1`/`sha2`/`digest`, `zip`, `tempfile`, Cargo integration tests.

---

## Phase File Map

- Create `src/loader/common.rs` for shared loader request/version types.
- Create `src/loader/fabric.rs` and `src/loader/quilt.rs` for metadata-profile loaders.
- Create `src/loader/forge.rs` and `src/loader/neoforge.rs` for Maven metadata and installer URL helpers.
- Rework `src/forge/mod.rs` as a compatibility wrapper around `loader::forge`.
- Create `src/install/loader.rs` for loader profile paths and writes.
- Modify `src/launcher.rs` to install Fabric/Quilt profile loaders and Forge/NeoForge installer loaders through the facade.
- Add loader fixture files under `tests/fixtures/`.
- Add `tests/loader_fabric_quilt.rs`, `tests/loader_forge_neoforge.rs`, and `tests/loader_install_integration.rs`.

## Task 7: Loader Metadata Providers For Fabric And Quilt

**Files:**
- Create: `src/loader/fabric.rs`
- Create: `src/loader/quilt.rs`
- Modify: `src/loader/mod.rs`
- Test: `tests/loader_fabric_quilt.rs`
- Fixtures: `tests/fixtures/fabric_loaders.json`, `tests/fixtures/fabric_profile_1_20_4.json`, `tests/fixtures/quilt_loaders.json`, `tests/fixtures/quilt_profile_1_20_4.json`

- [x] **Step 1: Add fixtures and failing tests**

Create `tests/fixtures/fabric_loaders.json`:

```json
[
  {"separator":".","build":2,"maven":"net.fabricmc:fabric-loader:0.19.2","version":"0.19.2","stable":true},
  {"separator":".","build":1,"maven":"net.fabricmc:fabric-loader:0.19.1","version":"0.19.1","stable":false}
]
```

Create `tests/fixtures/quilt_loaders.json`:

```json
[
  {"maven":"org.quiltmc:quilt-loader:0.30.0-beta.7","version":"0.30.0-beta.7","build":7,"separator":".","file_size":3105471,"hashes":{"sha1":"03400329bebe2445ea9d102f18adeffe368a0d67"}},
  {"maven":"org.quiltmc:quilt-loader:0.29.4","version":"0.29.4","build":4,"separator":".","file_size":3105471,"hashes":{"sha1":"1111111111111111111111111111111111111111"}}
]
```

Create `tests/fixtures/fabric_profile_1_20_4.json` using the Fabric profile shape:

```json
{
  "id": "fabric-loader-0.15.7-1.20.4",
  "inheritsFrom": "1.20.4",
  "type": "release",
  "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
  "minimumLauncherVersion": 21,
  "arguments": {
    "game": [],
    "jvm": ["-DFabricMcEmu= net.minecraft.client.main.Main "]
  },
  "libraries": [
    {"name":"net.fabricmc:intermediary:1.20.4","url":"https://maven.fabricmc.net/"},
    {"name":"net.fabricmc:fabric-loader:0.15.7","url":"https://maven.fabricmc.net/"}
  ]
}
```

Create `tests/fixtures/quilt_profile_1_20_4.json`:

```json
{
  "id": "quilt-loader-0.23.1-1.20.4",
  "inheritsFrom": "1.20.4",
  "type": "release",
  "mainClass": "org.quiltmc.loader.impl.launch.knot.KnotClient",
  "minimumLauncherVersion": 21,
  "arguments": {"game": []},
  "libraries": [
    {"name":"org.quiltmc:quilt-loader:0.23.1","url":"https://maven.quiltmc.org/repository/release/"},
    {"name":"net.fabricmc:intermediary:1.20.4","url":"https://maven.fabricmc.net/"}
  ]
}
```

Create `tests/loader_fabric_quilt.rs`:

```rust
use mc_launcher_core::{
    core::version::VersionJson,
    loader::{
        fabric::{FabricLoaderVersion, latest_stable_loader as latest_fabric_stable},
        quilt::{QuiltLoaderVersion, latest_loader as latest_quilt_loader},
    },
};

#[test]
fn fabric_latest_stable_prefers_stable_flag() {
    let versions: Vec<FabricLoaderVersion> =
        serde_json::from_str(include_str!("fixtures/fabric_loaders.json")).unwrap();
    assert_eq!(latest_fabric_stable(&versions).unwrap().version, "0.19.2");
}

#[test]
fn quilt_latest_loader_uses_first_entry() {
    let versions: Vec<QuiltLoaderVersion> =
        serde_json::from_str(include_str!("fixtures/quilt_loaders.json")).unwrap();
    assert_eq!(latest_quilt_loader(&versions).unwrap().version, "0.30.0-beta.7");
}

#[test]
fn fabric_profile_parses_as_version_json() {
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/fabric_profile_1_20_4.json")).unwrap();
    assert_eq!(profile.id.as_deref(), Some("fabric-loader-0.15.7-1.20.4"));
    assert_eq!(profile.inherits_from.as_deref(), Some("1.20.4"));
}

#[test]
fn quilt_profile_parses_as_version_json() {
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/quilt_profile_1_20_4.json")).unwrap();
    assert_eq!(profile.id.as_deref(), Some("quilt-loader-0.23.1-1.20.4"));
    assert_eq!(profile.inherits_from.as_deref(), Some("1.20.4"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test loader_fabric_quilt
```

Expected: FAIL because provider modules are missing.

- [x] **Step 3: Implement Fabric provider helpers**

Create `src/loader/fabric.rs`:

```rust
use serde::Deserialize;

use crate::{core::version::VersionJson, net::http, LauncherError, Result};

const FABRIC_META_BASE: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

pub fn latest_stable_loader(versions: &[FabricLoaderVersion]) -> Result<&FabricLoaderVersion> {
    versions.iter().find(|version| version.stable).ok_or_else(|| {
        LauncherError::LoaderVersionNotFound {
            loader: crate::loader::LoaderKind::Fabric,
            version: "latest stable".to_string(),
        }
    })
}

pub fn list_loader_versions() -> Result<Vec<FabricLoaderVersion>> {
    http::get_json(&format!("{FABRIC_META_BASE}/versions/loader"))
}

pub fn fetch_profile(minecraft_version: &str, loader_version: &str) -> Result<VersionJson> {
    http::get_json(&format!(
        "{FABRIC_META_BASE}/versions/loader/{minecraft_version}/{loader_version}/profile/json"
    ))
}
```

- [x] **Step 4: Implement Quilt provider helpers**

Create `src/loader/quilt.rs`:

```rust
use std::collections::HashMap;

use serde::Deserialize;

use crate::{core::version::VersionJson, net::http, LauncherError, Result};

const QUILT_META_BASE: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct QuiltLoaderVersion {
    pub maven: String,
    pub version: String,
    pub build: i32,
    pub separator: String,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub hashes: HashMap<String, String>,
}

pub fn latest_loader(versions: &[QuiltLoaderVersion]) -> Result<&QuiltLoaderVersion> {
    versions.first().ok_or_else(|| LauncherError::LoaderVersionNotFound {
        loader: crate::loader::LoaderKind::Quilt,
        version: "latest".to_string(),
    })
}

pub fn list_loader_versions() -> Result<Vec<QuiltLoaderVersion>> {
    http::get_json(&format!("{QUILT_META_BASE}/versions/loader"))
}

pub fn fetch_profile(minecraft_version: &str, loader_version: &str) -> Result<VersionJson> {
    http::get_json(&format!(
        "{QUILT_META_BASE}/versions/loader/{minecraft_version}/{loader_version}/profile/json"
    ))
}
```

Modify `src/loader/mod.rs`:

```rust
pub mod common;
pub mod fabric;
pub mod quilt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}
```

- [x] **Step 5: Run focused tests**

Run:

```bash
cargo test --test loader_fabric_quilt
```

Expected: PASS.

- [x] **Step 6: Commit**

```bash
git add src/loader tests/fixtures/fabric_loaders.json tests/fixtures/fabric_profile_1_20_4.json tests/fixtures/quilt_loaders.json tests/fixtures/quilt_profile_1_20_4.json tests/loader_fabric_quilt.rs
git commit -m "feat: add fabric and quilt metadata providers"
```

## Task 8: Forge And NeoForge Metadata Providers

**Files:**
- Create: `src/loader/forge.rs`
- Create: `src/loader/neoforge.rs`
- Modify: `src/loader/mod.rs`
- Modify: `src/forge/mod.rs`
- Test: `tests/loader_forge_neoforge.rs`
- Fixtures: `tests/fixtures/forge_maven_metadata.xml`, `tests/fixtures/neoforge_maven_metadata.xml`

- [x] **Step 1: Add fixtures and failing tests**

Create `tests/fixtures/forge_maven_metadata.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<metadata>
  <groupId>net.minecraftforge</groupId>
  <artifactId>forge</artifactId>
  <versioning>
    <latest>1.21.4-54.1.6</latest>
    <release>1.21.4-54.1.6</release>
    <versions>
      <version>1.20.4-49.0.50</version>
      <version>1.21.4-54.1.6</version>
    </versions>
  </versioning>
</metadata>
```

Create `tests/fixtures/neoforge_maven_metadata.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<metadata>
  <groupId>net.neoforged</groupId>
  <artifactId>neoforge</artifactId>
  <versioning>
    <latest>21.4.150</latest>
    <release>21.4.150</release>
    <versions>
      <version>20.4.240</version>
      <version>21.4.150</version>
    </versions>
  </versioning>
</metadata>
```

Create `tests/loader_forge_neoforge.rs`:

```rust
use mc_launcher_core::loader::{
    forge::{forge_installed_version_id, parse_maven_metadata as parse_forge_metadata},
    neoforge::{neoforge_installed_version_id, parse_maven_metadata as parse_neoforge_metadata},
};

#[test]
fn parses_forge_maven_metadata() {
    let metadata = parse_forge_metadata(include_str!("fixtures/forge_maven_metadata.xml")).unwrap();
    assert_eq!(metadata.latest, "1.21.4-54.1.6");
    assert_eq!(metadata.versions, vec!["1.20.4-49.0.50", "1.21.4-54.1.6"]);
}

#[test]
fn maps_forge_version_to_installed_id() {
    assert_eq!(
        forge_installed_version_id("1.20.4-49.0.50").unwrap(),
        "1.20.4-forge-49.0.50"
    );
}

#[test]
fn parses_neoforge_maven_metadata() {
    let metadata =
        parse_neoforge_metadata(include_str!("fixtures/neoforge_maven_metadata.xml")).unwrap();
    assert_eq!(metadata.latest, "21.4.150");
    assert_eq!(metadata.versions, vec!["20.4.240", "21.4.150"]);
}

#[test]
fn maps_neoforge_version_to_installed_id() {
    assert_eq!(
        neoforge_installed_version_id("1.21.4", "21.4.150"),
        "neoforge-21.4.150"
    );
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test loader_forge_neoforge
```

Expected: FAIL because Forge and NeoForge modules are missing.

- [x] **Step 3: Implement shared Maven metadata parser in Forge module**

Create `src/loader/forge.rs`:

```rust
use regex::Regex;

use crate::{net::http, LauncherError, Result};

const FORGE_METADATA_URL: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenMetadata {
    pub latest: String,
    pub release: String,
    pub versions: Vec<String>,
}

pub fn parse_maven_metadata(xml: &str) -> Result<MavenMetadata> {
    let latest = capture_one(xml, r"<latest>(.*?)</latest>", "latest")?;
    let release = capture_one(xml, r"<release>(.*?)</release>", "release")?;
    let version_re = Regex::new(r"<version>(.*?)</version>").map_err(|err| LauncherError::Other {
        message: err.to_string(),
    })?;
    let versions = version_re
        .captures_iter(xml)
        .filter_map(|captures| captures.get(1).map(|m| m.as_str().to_string()))
        .collect();
    Ok(MavenMetadata {
        latest,
        release,
        versions,
    })
}

fn capture_one(xml: &str, pattern: &str, field: &str) -> Result<String> {
    let re = Regex::new(pattern).map_err(|err| LauncherError::Other {
        message: err.to_string(),
    })?;
    re.captures(xml)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| LauncherError::MissingField {
            context: "maven metadata".to_string(),
            field: field.to_string(),
        })
}

pub fn list_forge_versions() -> Result<Vec<String>> {
    Ok(parse_maven_metadata(&http::get_text(FORGE_METADATA_URL)?)?.versions)
}

pub fn forge_installed_version_id(forge_version: &str) -> Result<String> {
    let Some((minecraft, forge)) = forge_version.split_once('-') else {
        return Err(LauncherError::InvalidVersionId {
            id: forge_version.to_string(),
        });
    };
    Ok(format!("{minecraft}-forge-{forge}"))
}

pub fn installer_url(forge_version: &str) -> String {
    format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{0}/forge-{0}-installer.jar",
        forge_version
    )
}
```

- [x] **Step 4: Implement NeoForge module**

Create `src/loader/neoforge.rs`:

```rust
use crate::{loader::forge::MavenMetadata, net::http, Result};

const NEOFORGE_METADATA_URL: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

pub fn parse_maven_metadata(xml: &str) -> Result<MavenMetadata> {
    crate::loader::forge::parse_maven_metadata(xml)
}

pub fn list_neoforge_versions() -> Result<Vec<String>> {
    Ok(parse_maven_metadata(&http::get_text(NEOFORGE_METADATA_URL)?)?.versions)
}

pub fn neoforge_installed_version_id(_minecraft_version: &str, neoforge_version: &str) -> String {
    format!("neoforge-{neoforge_version}")
}

pub fn installer_url(neoforge_version: &str) -> String {
    format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{0}/neoforge-{0}-installer.jar",
        neoforge_version
    )
}
```

Modify `src/loader/mod.rs`:

```rust
pub mod common;
pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod quilt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}
```

Replace `src/forge/mod.rs` with compatibility exports:

```rust
use std::path::Path;

use crate::{loader::forge, Result};

#[deprecated(note = "use loader::forge::list_forge_versions")]
pub fn list_forge_versions() -> Result<Vec<String>> {
    forge::list_forge_versions()
}

#[deprecated(note = "use loader::forge::forge_installed_version_id")]
pub fn forge_to_installed_version(forge_version: &str) -> Result<String> {
    forge::forge_installed_version_id(forge_version)
}

#[deprecated(note = "use loader::forge installer support through Launcher::install")]
pub fn run_forge_installer(
    version: &str,
    _java: Option<impl AsRef<Path>>,
) -> Result<()> {
    Err(crate::LauncherError::Other {
        message: format!(
            "direct Forge installer execution for {version} moved to Launcher::install"
        ),
    })
}
```

- [x] **Step 5: Run focused tests**

Run:

```bash
cargo test --test loader_forge_neoforge
```

Expected: PASS.

- [x] **Step 6: Commit**

```bash
git add src/loader src/forge/mod.rs tests/fixtures/forge_maven_metadata.xml tests/fixtures/neoforge_maven_metadata.xml tests/loader_forge_neoforge.rs
git commit -m "feat: add forge and neoforge metadata providers"
```

## Task 9: Loader Install Integration

**Files:**
- Modify: `src/install/request.rs`
- Create: `src/install/loader.rs`
- Modify: `src/install/mod.rs`
- Modify: `src/launcher.rs`
- Test: `tests/loader_install_integration.rs`

- [x] **Step 1: Write failing tests for installing loader profile fixtures**

Create `tests/loader_install_integration.rs`:

```rust
use std::fs;

use mc_launcher_core::{
    core::version::VersionJson,
    install::loader::{
        installer_command_args, loader_profile_path, write_loader_profile, InstallerInvocation,
    },
    loader::LoaderKind,
};

#[test]
fn writes_loader_profile_to_versions_directory() {
    let dir = tempfile::tempdir().unwrap();
    let profile: VersionJson =
        serde_json::from_str(include_str!("fixtures/fabric_profile_1_20_4.json")).unwrap();

    let path = write_loader_profile(dir.path(), &profile).unwrap();

    assert_eq!(
        path,
        dir.path()
            .join("versions")
            .join("fabric-loader-0.15.7-1.20.4")
            .join("fabric-loader-0.15.7-1.20.4.json")
    );
    assert!(path.is_file());

    let saved: VersionJson = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(saved.id.as_deref(), Some("fabric-loader-0.15.7-1.20.4"));
}

#[test]
fn computes_loader_profile_path() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        loader_profile_path(dir.path(), "quilt-loader-0.23.1-1.20.4"),
        dir.path()
            .join("versions")
            .join("quilt-loader-0.23.1-1.20.4")
            .join("quilt-loader-0.23.1-1.20.4.json")
    );
}

#[test]
fn builds_installer_command_arguments() {
    let invocation = InstallerInvocation {
        loader: LoaderKind::Forge,
        java_executable: "java".into(),
        installer_path: "/tmp/forge-installer.jar".into(),
        minecraft_dir: "/tmp/mc".into(),
    };

    assert_eq!(
        installer_command_args(&invocation),
        vec![
            "-jar".to_string(),
            "/tmp/forge-installer.jar".to_string(),
            "--installClient".to_string(),
            "/tmp/mc".to_string()
        ]
    );
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test loader_install_integration
```

Expected: FAIL because install loader helpers are missing.

- [x] **Step 3: Implement loader profile writing**

Create `src/install/loader.rs`:

```rust
use std::{
    fs,
    process::Command,
    path::{Path, PathBuf},
};

use crate::{core::version::VersionJson, loader::LoaderKind, LauncherError, Result};

pub fn loader_profile_path(minecraft_dir: impl AsRef<Path>, version_id: &str) -> PathBuf {
    minecraft_dir
        .as_ref()
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"))
}

pub fn write_loader_profile(minecraft_dir: impl AsRef<Path>, profile: &VersionJson) -> Result<PathBuf> {
    let version_id = profile.id.as_deref().ok_or_else(|| LauncherError::MissingField {
        context: "loader profile".to_string(),
        field: "id".to_string(),
    })?;
    let path = loader_profile_path(minecraft_dir, version_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_vec_pretty(profile)?)?;
    Ok(path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerInvocation {
    pub loader: LoaderKind,
    pub java_executable: PathBuf,
    pub installer_path: PathBuf,
    pub minecraft_dir: PathBuf,
}

pub fn installer_command_args(invocation: &InstallerInvocation) -> Vec<String> {
    vec![
        "-jar".to_string(),
        invocation.installer_path.to_string_lossy().to_string(),
        "--installClient".to_string(),
        invocation.minecraft_dir.to_string_lossy().to_string(),
    ]
}

pub fn run_loader_installer(invocation: &InstallerInvocation) -> Result<()> {
    let status = Command::new(&invocation.java_executable)
        .args(installer_command_args(invocation))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(LauncherError::InstallerFailed {
            loader: invocation.loader,
            status: status.code(),
        })
    }
}
```

Modify `src/install/mod.rs`:

```rust
pub mod assets;
pub mod libraries;
pub mod loader;
pub mod request;
pub mod vanilla;

pub use request::{InstallRequest, InstallResult, JavaInstallPolicy};
```

- [x] **Step 4: Extend facade install for metadata-profile and installer loaders**

Modify `src/launcher.rs` so `install` handles Fabric and Quilt by resolving loader versions online and writing profiles, and handles Forge and NeoForge by downloading and running installer JARs:

```rust
pub fn install(&self, request: InstallRequest) -> Result<InstallResult> {
    if let Some(loader) = request.loader {
        match loader {
            crate::loader::common::LoaderSpec::Fabric { version } => {
                let loader_version = resolve_fabric_loader_version(version)?;
                let profile = crate::loader::fabric::fetch_profile(&request.minecraft_version, &loader_version)?;
                crate::install::loader::write_loader_profile(&self.minecraft_dir, &profile)?;
                let version_id = profile.id.ok_or_else(|| crate::LauncherError::MissingField {
                    context: "loader profile".to_string(),
                    field: "id".to_string(),
                })?;
                return Ok(InstallResult { version_id });
            }
            crate::loader::common::LoaderSpec::Quilt { version } => {
                let loader_version = resolve_quilt_loader_version(version)?;
                let profile = crate::loader::quilt::fetch_profile(&request.minecraft_version, &loader_version)?;
                crate::install::loader::write_loader_profile(&self.minecraft_dir, &profile)?;
                let version_id = profile.id.ok_or_else(|| crate::LauncherError::MissingField {
                    context: "loader profile".to_string(),
                    field: "id".to_string(),
                })?;
                return Ok(InstallResult { version_id });
            }
            crate::loader::common::LoaderSpec::Forge { version } => {
                let loader_version = resolve_forge_loader_version(version)?;
                let installer_path = download_installer(
                    &self.minecraft_dir,
                    "forge",
                    &loader_version,
                    &crate::loader::forge::installer_url(&loader_version),
                )?;
                crate::install::loader::run_loader_installer(&crate::install::loader::InstallerInvocation {
                    loader: crate::loader::LoaderKind::Forge,
                    java_executable: std::path::PathBuf::from("java"),
                    installer_path,
                    minecraft_dir: self.minecraft_dir.clone(),
                })?;
                return Ok(InstallResult {
                    version_id: crate::loader::forge::forge_installed_version_id(&loader_version)?,
                });
            }
            crate::loader::common::LoaderSpec::NeoForge { version } => {
                let loader_version = resolve_neoforge_loader_version(version)?;
                let installer_path = download_installer(
                    &self.minecraft_dir,
                    "neoforge",
                    &loader_version,
                    &crate::loader::neoforge::installer_url(&loader_version),
                )?;
                crate::install::loader::run_loader_installer(&crate::install::loader::InstallerInvocation {
                    loader: crate::loader::LoaderKind::NeoForge,
                    java_executable: std::path::PathBuf::from("java"),
                    installer_path,
                    minecraft_dir: self.minecraft_dir.clone(),
                })?;
                return Ok(InstallResult {
                    version_id: crate::loader::neoforge::neoforge_installed_version_id(
                        &request.minecraft_version,
                        &loader_version,
                    ),
                });
            }
        }
    }

    Ok(InstallResult {
        version_id: request.minecraft_version,
    })
}
```

Add private helper functions in `src/launcher.rs`:

```rust
fn resolve_fabric_loader_version(version: crate::loader::common::LoaderVersion) -> Result<String> {
    match version {
        crate::loader::common::LoaderVersion::Exact(version) => Ok(version),
        crate::loader::common::LoaderVersion::Latest | crate::loader::common::LoaderVersion::LatestStable => {
            let versions = crate::loader::fabric::list_loader_versions()?;
            Ok(crate::loader::fabric::latest_stable_loader(&versions)?.version.clone())
        }
    }
}

fn resolve_quilt_loader_version(version: crate::loader::common::LoaderVersion) -> Result<String> {
    match version {
        crate::loader::common::LoaderVersion::Exact(version) => Ok(version),
        crate::loader::common::LoaderVersion::Latest | crate::loader::common::LoaderVersion::LatestStable => {
            let versions = crate::loader::quilt::list_loader_versions()?;
            Ok(crate::loader::quilt::latest_loader(&versions)?.version.clone())
        }
    }
}

fn resolve_forge_loader_version(version: crate::loader::common::LoaderVersion) -> Result<String> {
    match version {
        crate::loader::common::LoaderVersion::Exact(version) => Ok(version),
        crate::loader::common::LoaderVersion::Latest | crate::loader::common::LoaderVersion::LatestStable => {
            let versions = crate::loader::forge::list_forge_versions()?;
            versions.last().cloned().ok_or_else(|| crate::LauncherError::LoaderVersionNotFound {
                loader: crate::loader::LoaderKind::Forge,
                version: "latest".to_string(),
            })
        }
    }
}

fn resolve_neoforge_loader_version(version: crate::loader::common::LoaderVersion) -> Result<String> {
    match version {
        crate::loader::common::LoaderVersion::Exact(version) => Ok(version),
        crate::loader::common::LoaderVersion::Latest | crate::loader::common::LoaderVersion::LatestStable => {
            let versions = crate::loader::neoforge::list_neoforge_versions()?;
            versions.last().cloned().ok_or_else(|| crate::LauncherError::LoaderVersionNotFound {
                loader: crate::loader::LoaderKind::NeoForge,
                version: "latest".to_string(),
            })
        }
    }
}

fn download_installer(
    minecraft_dir: &std::path::Path,
    loader_name: &str,
    loader_version: &str,
    url: &str,
) -> Result<std::path::PathBuf> {
    let destination = minecraft_dir
        .join("versions")
        .join(".installers")
        .join(format!("{loader_name}-{loader_version}-installer.jar"));
    let plan = crate::net::download::DownloadPlan {
        tasks: vec![crate::net::download::DownloadTask {
            url: url.to_string(),
            destination: destination.clone(),
            checksum: None,
            label: format!("{loader_name} installer {loader_version}"),
        }],
    };
    let mut reporter = |_event: crate::progress::ProgressEvent| {};
    crate::net::download::execute_plan(&plan, &mut reporter)?;
    Ok(destination)
}
```

- [x] **Step 5: Run focused tests**

Run:

```bash
cargo test --test loader_install_integration
```

Expected: PASS.

- [x] **Step 6: Commit**

```bash
git add src/install src/launcher.rs tests/loader_install_integration.rs
git commit -m "feat: integrate loader profile installation"
```
