# Launcher Core Redesign Phase 1: Foundation And Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modernize dependencies, restore cross-platform compilation, and add typed core primitives.

**Architecture:** This phase fixes the known macOS compile failure first, then introduces typed errors, platform detection, Maven parsing, hashing, path safety, rule evaluation, and version inheritance. It produces testable pure core behavior before install or loader work depends on it.

**Tech Stack:** Rust 2021, `reqwest` blocking client, `serde`, `serde_json`, `thiserror`, `once_cell`, RustCrypto `sha1`/`sha2`/`digest`, `zip`, `tempfile`, Cargo integration tests.

---

## Phase File Map

- Modify `Cargo.toml` and `Cargo.lock` to modernize dependencies and fix target-specific Windows handling.
- Modify `src/lib.rs` to expose new modules and the crate-level `Result` alias.
- Create `src/error.rs` for `LauncherError`.
- Create `src/platform.rs` for OS and architecture detection.
- Create `src/loader/mod.rs` as the initial loader kind scaffold needed by errors.
- Create `src/core/maven.rs`, `src/core/rules.rs`, and `src/core/version.rs` for pure domain behavior.
- Create `src/io/hash.rs` and `src/io/paths.rs` for checksum and path-safety helpers.
- Add `tests/error_baseline.rs`, `tests/core_maven_io.rs`, and `tests/core_rules_version.rs`.

## Task 1: Dependency Baseline And Typed Error Skeleton

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/platform.rs`
- Test: `tests/error_baseline.rs`

- [x] **Step 1: Write the failing typed error smoke test**

Create `tests/error_baseline.rs`:

```rust
use mc_launcher_core::{LauncherError, Result};

fn returns_error() -> Result<()> {
    Err(LauncherError::InvalidVersionId {
        id: "bad/version".to_string(),
    })
}

#[test]
fn launcher_error_formats_context() {
    let err = returns_error().unwrap_err();
    assert!(err.to_string().contains("bad/version"));
}
```

- [x] **Step 2: Run the test and confirm the current crate still fails before the fix**

Run:

```bash
cargo test --test error_baseline
```

Expected: FAIL before implementation. On macOS the failure may be the current `winver` compile error; that still validates the task starts from the known broken baseline.

- [x] **Step 3: Update dependencies**

Edit `Cargo.toml` so the dependency section contains these direct dependencies:

```toml
[dependencies]
base64 = "0.22.1"
chrono = "0.4.44"
once_cell = "1.21.4"
rand = "0.9.2"
regex = "1.12.3"
reqwest = { version = "0.13.3", features = ["blocking", "json", "rustls"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.150"
serde_urlencoded = "0.7.1"
sha1 = "0.11.0"
sha2 = "0.11.0"
sysinfo = "0.39.2"
tempfile = "3.27.0"
thiserror = "2.0.18"
url = "2.5.8"
uuid = { version = "1.23.1", features = ["v4", "fast-rng", "macro-diagnostics"] }
which = "8.0.2"
xz2 = "0.1.7"
zip = "2.4.2"

[target.'cfg(windows)'.dependencies]
winver = "1.0.0"
```

Remove direct `lazy_static`, `ring`, and `rust-crypto` entries. Keep the existing package metadata.

- [x] **Step 4: Add the error module**

Create `src/error.rs`:

```rust
use std::path::PathBuf;

use crate::loader::LoaderKind;

pub type Result<T> = std::result::Result<T, LauncherError>;

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("network error: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },
    #[error("io error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("json error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
    #[error("invalid version id: {id}")]
    InvalidVersionId { id: String },
    #[error("unsupported platform: {os}/{arch}")]
    UnsupportedPlatform { os: String, arch: String },
    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    #[error("unsafe path {path} escapes base {base}")]
    UnsafePath { base: PathBuf, path: PathBuf },
    #[error("invalid maven coordinate: {coordinate}")]
    InvalidMavenCoordinate { coordinate: String },
    #[error("{loader:?} loader version not found: {version}")]
    LoaderVersionNotFound { loader: LoaderKind, version: String },
    #[error("{loader:?} installer failed with status {status:?}")]
    InstallerFailed {
        loader: LoaderKind,
        status: Option<i32>,
    },
    #[error("missing field {field} in {context}")]
    MissingField { context: String, field: String },
    #[error("{message}")]
    Other { message: String },
}
```

- [x] **Step 5: Add the platform module**

Create `src/platform.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86,
    X86_64,
    Aarch64,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    pub fn current() -> Self {
        Self {
            os: match std::env::consts::OS {
                "windows" => Os::Windows,
                "macos" => Os::MacOs,
                "linux" => Os::Linux,
                _ => Os::Other,
            },
            arch: match std::env::consts::ARCH {
                "x86" | "i386" | "i586" | "i686" => Arch::X86,
                "x86_64" | "amd64" => Arch::X86_64,
                "aarch64" => Arch::Aarch64,
                _ => Arch::Other,
            },
        }
    }

    pub fn minecraft_os_name(self) -> &'static str {
        match self.os {
            Os::Windows => "windows",
            Os::MacOs => "osx",
            Os::Linux => "linux",
            Os::Other => "unknown",
        }
    }

    pub fn is_32_bit(self) -> bool {
        self.arch == Arch::X86
    }
}
```

- [x] **Step 6: Add temporary loader kind and exports**

Create `src/loader/mod.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}
```

Modify `src/lib.rs`:

```rust
pub mod account;
pub mod auth;
pub mod command;
pub mod core;
pub mod error;
pub mod forge;
pub mod install;
pub mod io;
pub mod launcher;
pub mod loader;
pub mod net;
pub mod platform;
pub mod prelude;
pub mod progress;
pub mod types;
pub mod utils;

pub use error::{LauncherError, Result};
```

Create empty module files needed by these exports with module comments so the crate compiles:

```rust
//! Module scaffold; concrete behavior is added by later plan tasks.
```

Use this scaffold for `src/account.rs`, `src/core/mod.rs`, `src/io/mod.rs`, `src/launcher.rs`, `src/net/mod.rs`, `src/prelude.rs`, and `src/progress.rs`.

- [x] **Step 7: Run the focused test**

Run:

```bash
cargo test --test error_baseline
```

Expected: PASS.

- [x] **Step 8: Run full compile checks**

Run:

```bash
cargo test
cargo test --examples
```

Expected: Existing code may now fail due to dependency API changes. Fix mechanical API changes in existing modules without changing behavior. The final expected result for this task is PASS for both commands.

- [x] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock src tests/error_baseline.rs
git commit -m "chore: modernize dependencies and add launcher error"
```

## Task 2: Core Maven, Hashing, And Path Safety

**Files:**
- Create: `src/core/maven.rs`
- Modify: `src/core/mod.rs`
- Create: `src/io/hash.rs`
- Create: `src/io/paths.rs`
- Modify: `src/io/mod.rs`
- Test: `tests/core_maven_io.rs`

- [x] **Step 1: Write failing tests for Maven paths, hashes, and safe paths**

Create `tests/core_maven_io.rs`:

```rust
use std::fs;

use mc_launcher_core::{
    core::maven::MavenCoordinate,
    io::{hash::sha1_file, paths::safe_join},
};

#[test]
fn parses_maven_coordinate_with_classifier() {
    let coordinate = MavenCoordinate::parse("org.lwjgl:lwjgl:3.3.2:natives-macos-arm64").unwrap();
    assert_eq!(coordinate.group, "org.lwjgl");
    assert_eq!(coordinate.artifact, "lwjgl");
    assert_eq!(coordinate.version, "3.3.2");
    assert_eq!(coordinate.classifier.as_deref(), Some("natives-macos-arm64"));
    assert_eq!(coordinate.extension, "jar");
    assert_eq!(
        coordinate.artifact_path().to_string_lossy(),
        "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-macos-arm64.jar"
    );
}

#[test]
fn parses_maven_coordinate_with_extension() {
    let coordinate = MavenCoordinate::parse("com.example:demo:1.0@zip").unwrap();
    assert_eq!(coordinate.extension, "zip");
    assert_eq!(
        coordinate.artifact_path().to_string_lossy(),
        "com/example/demo/1.0/demo-1.0.zip"
    );
}

#[test]
fn rejects_invalid_maven_coordinate() {
    let err = MavenCoordinate::parse("bad").unwrap_err();
    assert!(err.to_string().contains("invalid maven coordinate"));
}

#[test]
fn computes_sha1_for_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("sample.txt");
    fs::write(&file, b"minecraft").unwrap();

    assert_eq!(
        sha1_file(&file).unwrap(),
        "d33f8bf35eac3e8b9f5b5b86433f62d4bda5c9f1"
    );
}

#[test]
fn safe_join_rejects_parent_escape() {
    let dir = tempfile::tempdir().unwrap();
    let err = safe_join(dir.path(), "../escape.jar").unwrap_err();
    assert!(err.to_string().contains("unsafe path"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test core_maven_io
```

Expected: FAIL because modules/functions are missing.

- [x] **Step 3: Implement Maven coordinate parsing**

Create `src/core/maven.rs`:

```rust
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
            [group, artifact, version] if !group.is_empty() && !artifact.is_empty() && !version.is_empty() => {
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
```

Modify `src/core/mod.rs`:

```rust
pub mod maven;
```

- [x] **Step 4: Implement hashing**

Create `src/io/hash.rs`:

```rust
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use sha1::{Digest, Sha1};

use crate::Result;

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

    Ok(format!("{:x}", hasher.finalize()))
}
```

- [x] **Step 5: Implement safe path joining**

Create `src/io/paths.rs`:

```rust
use std::{
    path::{Component, Path, PathBuf},
};

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
```

Modify `src/io/mod.rs`:

```rust
pub mod hash;
pub mod paths;
```

- [x] **Step 6: Run the focused tests**

Run:

```bash
cargo test --test core_maven_io
```

Expected: PASS.

- [x] **Step 7: Commit**

```bash
git add src/core src/io tests/core_maven_io.rs
git commit -m "feat: add maven parsing and safe io helpers"
```

## Task 3: Rules, Version Models, And Inheritance

**Files:**
- Create: `src/core/rules.rs`
- Create: `src/core/version.rs`
- Modify: `src/core/mod.rs`
- Test: `tests/core_rules_version.rs`

- [x] **Step 1: Write failing tests for rules and inheritance**

Create `tests/core_rules_version.rs`:

```rust
use mc_launcher_core::{
    core::{
        rules::{FeatureSet, Rule, RuleAction, RuleOs, evaluate_rules},
        version::VersionJson,
    },
    platform::{Arch, Os, Platform},
};

#[test]
fn allows_matching_os_rule() {
    let rules = vec![Rule {
        action: RuleAction::Allow,
        os: Some(RuleOs {
            name: Some("osx".to_string()),
            arch: None,
            version: None,
        }),
        features: None,
    }];

    assert!(evaluate_rules(
        &rules,
        Platform {
            os: Os::MacOs,
            arch: Arch::Aarch64,
        },
        &FeatureSet::default()
    ));
}

#[test]
fn rejects_non_matching_os_rule() {
    let rules = vec![Rule {
        action: RuleAction::Allow,
        os: Some(RuleOs {
            name: Some("windows".to_string()),
            arch: None,
            version: None,
        }),
        features: None,
    }];

    assert!(!evaluate_rules(
        &rules,
        Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        },
        &FeatureSet::default()
    ));
}

#[test]
fn child_version_overrides_main_class_and_extends_libraries() {
    let parent: VersionJson = serde_json::from_str(
        r#"{
            "id":"1.20.4",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "minimumLauncherVersion":21,
            "libraries":[{"name":"com.example:parent:1.0"}],
            "arguments":{"game":["--username","${auth_player_name}"],"jvm":["-cp","${classpath}"]}
        }"#,
    )
    .unwrap();

    let child: VersionJson = serde_json::from_str(
        r#"{
            "id":"fabric-loader-0.15.7-1.20.4",
            "inheritsFrom":"1.20.4",
            "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
            "minimumLauncherVersion":21,
            "libraries":[{"name":"net.fabricmc:fabric-loader:0.15.7"}],
            "arguments":{"game":[],"jvm":["-DFabricMcEmu= net.minecraft.client.main.Main "]}
        }"#,
    )
    .unwrap();

    let merged = parent.merge_child(&child);
    assert_eq!(
        merged.main_class.as_deref(),
        Some("net.fabricmc.loader.impl.launch.knot.KnotClient")
    );
    assert_eq!(merged.libraries.len(), 2);
    assert_eq!(merged.arguments.jvm.len(), 3);
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test core_rules_version
```

Expected: FAIL because rule and version models are missing.

- [x] **Step 3: Implement rules**

Create `src/core/rules.rs` with serde-compatible rule models:

```rust
use std::collections::HashMap;

use serde::Deserialize;

use crate::platform::{Arch, Platform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RuleOs {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<RuleOs>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FeatureSet {
    pub demo_user: bool,
    pub custom_resolution: bool,
    pub quick_play: bool,
    pub quick_play_singleplayer: bool,
    pub quick_play_multiplayer: bool,
    pub quick_play_realms: bool,
}

pub fn evaluate_rules(rules: &[Rule], platform: Platform, features: &FeatureSet) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule, platform, features) {
            allowed = rule.action == RuleAction::Allow;
        }
    }
    allowed
}

fn rule_matches(rule: &Rule, platform: Platform, features: &FeatureSet) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != platform.minecraft_os_name() {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            if arch == "x86" && platform.arch != Arch::X86 {
                return false;
            }
        }
    }

    if let Some(rule_features) = &rule.features {
        for (name, expected) in rule_features {
            let actual = match name.as_str() {
                "is_demo_user" => features.demo_user,
                "has_custom_resolution" => features.custom_resolution,
                "has_quick_plays_support" => features.quick_play,
                "is_quick_play_singleplayer" => features.quick_play_singleplayer,
                "is_quick_play_multiplayer" => features.quick_play_multiplayer,
                "is_quick_play_realms" => features.quick_play_realms,
                _ => false,
            };
            if actual != *expected {
                return false;
            }
        }
    }

    true
}
```

- [x] **Step 4: Implement version models and merge**

Create `src/core/version.rs` with fields needed by current install and command flows:

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::rules::Rule;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct VersionJson {
    pub id: Option<String>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "minimumLauncherVersion", default)]
    pub minimum_launcher_version: Option<i32>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    #[serde(default)]
    pub downloads: HashMap<String, DownloadInfo>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub arguments: Arguments,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub logging: HashMap<String, LoggingConfig>,
    pub jar: Option<String>,
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    Ruled {
        #[serde(default)]
        rules: Vec<Rule>,
        value: StringOrVec,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Library {
    pub name: String,
    pub url: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    #[serde(default)]
    pub classifiers: HashMap<String, LibraryArtifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LibraryArtifact {
    pub path: String,
    pub url: String,
    pub sha1: String,
    pub size: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingConfig {
    pub argument: String,
    pub file: LoggingFile,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

impl VersionJson {
    pub fn merge_child(mut self, child: &VersionJson) -> VersionJson {
        self.id = child.id.clone().or(self.id);
        self.inherits_from = child.inherits_from.clone().or(self.inherits_from);
        self.r#type = child.r#type.clone().or(self.r#type);
        self.main_class = child.main_class.clone().or(self.main_class);
        self.minimum_launcher_version = child.minimum_launcher_version.or(self.minimum_launcher_version);
        self.assets = child.assets.clone().or(self.assets);
        self.asset_index = child.asset_index.clone().or(self.asset_index);
        self.downloads.extend(child.downloads.clone());
        self.libraries.extend(child.libraries.clone());
        self.arguments.game.extend(child.arguments.game.clone());
        self.arguments.jvm.extend(child.arguments.jvm.clone());
        self.minecraft_arguments = child.minecraft_arguments.clone().or(self.minecraft_arguments);
        self.java_version = child.java_version.clone().or(self.java_version);
        self.logging.extend(child.logging.clone());
        self.jar = child.jar.clone().or(self.jar);
        self.release_time = child.release_time.clone().or(self.release_time);
        self.time = child.time.clone().or(self.time);
        self.compliance_level = child.compliance_level.or(self.compliance_level);
        self
    }
}
```

Modify `src/core/mod.rs`:

```rust
pub mod maven;
pub mod rules;
pub mod version;
```

- [x] **Step 5: Run focused tests**

Run:

```bash
cargo test --test core_rules_version
```

Expected: PASS.

- [x] **Step 6: Commit**

```bash
git add src/core tests/core_rules_version.rs
git commit -m "feat: add version rules and inheritance model"
```
