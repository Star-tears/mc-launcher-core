# Launcher Core Redesign Phase 2: Command And Install Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build command generation, download planning, safe extraction, vanilla install planning, and the Launcher facade.

**Architecture:** This phase consumes the core primitives from phase 1 and adds the first usable SDK shape. It keeps IO and network effects behind DownloadTask/DownloadPlan and adds the facade API without loader-specific complexity.

**Tech Stack:** Rust 2021, `reqwest` blocking client, `serde`, `serde_json`, `thiserror`, `once_cell`, RustCrypto `sha1`/`sha2`/`digest`, `zip`, `tempfile`, Cargo integration tests.

---

## Phase File Map

- Create `src/account.rs` for public account identity used by launch options.
- Create `src/core/arguments.rs` and `src/core/classpath.rs` for argument replacement and classpath construction.
- Create `src/command/builder.rs` and rework `src/command/mod.rs` around `LaunchCommand`.
- Create `src/progress.rs`, `src/net/http.rs`, and `src/net/download.rs` for structured progress and download plans.
- Create `src/io/archive.rs` for safe archive extraction.
- Create `src/install/request.rs`, `src/install/vanilla.rs`, `src/install/libraries.rs`, and `src/install/assets.rs` for vanilla planning.
- Create `src/launcher.rs` and `src/prelude.rs` for the first public facade shape.
- Add `tests/command_builder.rs`, `tests/download_archive.rs`, and `tests/install_plan.rs`.

## Task 4: Argument Evaluation And Launch Command Builder

**Files:**
- Create: `src/account.rs`
- Create: `src/core/arguments.rs`
- Create: `src/core/classpath.rs`
- Modify: `src/core/mod.rs`
- Create: `src/command/builder.rs`
- Modify: `src/command/mod.rs`
- Test: `tests/command_builder.rs`

- [x] **Step 1: Write failing command builder tests**

Create `tests/command_builder.rs`:

```rust
use std::path::PathBuf;

use mc_launcher_core::{
    account::Account,
    command::builder::{LaunchCommand, LaunchOptions, build_launch_command},
    core::version::VersionJson,
};

#[test]
fn builds_basic_modern_launch_command() {
    let version: VersionJson = serde_json::from_str(
        r#"{
            "id":"1.20.4",
            "type":"release",
            "mainClass":"net.minecraft.client.main.Main",
            "minimumLauncherVersion":21,
            "arguments":{
                "jvm":["-Djava.library.path=${natives_directory}","-cp","${classpath}"],
                "game":["--username","${auth_player_name}","--version","${version_name}","--gameDir","${game_directory}","--assetsDir","${assets_root}","--assetIndex","${assets_index_name}","--uuid","${auth_uuid}","--accessToken","${auth_access_token}","--userType","${user_type}","--versionType","${version_type}"]
            },
            "assets":"12",
            "libraries":[{"name":"com.example:demo:1.0"}]
        }"#,
    )
    .unwrap();

    let command = build_launch_command(
        &version,
        PathBuf::from("/tmp/mc"),
        LaunchOptions {
            account: Account::offline("Steve"),
            java_executable: Some(PathBuf::from("/usr/bin/java")),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(command.executable, PathBuf::from("/usr/bin/java"));
    assert!(command.args.contains(&"net.minecraft.client.main.Main".to_string()));
    assert!(command.args.contains(&"Steve".to_string()));
    assert!(command.args.iter().any(|arg| arg.contains("libraries/com/example/demo/1.0/demo-1.0.jar")));
}

#[test]
fn launch_command_exposes_process_parts() {
    let command = LaunchCommand {
        executable: PathBuf::from("java"),
        args: vec!["-version".to_string()],
        working_dir: PathBuf::from("/tmp/mc"),
        env: Vec::new(),
    };

    assert_eq!(command.to_process_parts().0, PathBuf::from("java"));
    assert_eq!(command.to_process_parts().1, vec!["-version".to_string()]);
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test command_builder
```

Expected: FAIL because account and command builder types are missing.

- [x] **Step 3: Add public account model**

Create `src/account.rs`:

```rust
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Account {
    Offline {
        username: String,
        uuid: String,
    },
    Microsoft {
        username: String,
        uuid: String,
        access_token: String,
    },
}

impl Account {
    pub fn offline(username: impl Into<String>) -> Self {
        Self::Offline {
            username: username.into(),
            uuid: Uuid::new_v4().to_string(),
        }
    }

    pub fn username(&self) -> &str {
        match self {
            Self::Offline { username, .. } | Self::Microsoft { username, .. } => username,
        }
    }

    pub fn uuid(&self) -> &str {
        match self {
            Self::Offline { uuid, .. } | Self::Microsoft { uuid, .. } => uuid,
        }
    }

    pub fn access_token(&self) -> &str {
        match self {
            Self::Offline { .. } => "",
            Self::Microsoft { access_token, .. } => access_token,
        }
    }
}
```

- [x] **Step 4: Implement argument replacement and classpath helpers**

Create `src/core/arguments.rs`:

```rust
use std::{collections::HashMap, path::Path};

use crate::{
    account::Account,
    core::{
        rules::{FeatureSet, evaluate_rules},
        version::{ArgumentValue, StringOrVec, VersionJson},
    },
    platform::Platform,
};

#[derive(Debug, Clone)]
pub struct ArgumentContext<'a> {
    pub minecraft_dir: &'a Path,
    pub natives_dir: &'a Path,
    pub game_dir: &'a Path,
    pub version: &'a VersionJson,
    pub account: &'a Account,
    pub classpath: &'a str,
    pub launcher_name: &'a str,
    pub launcher_version: &'a str,
    pub version_type: &'a str,
    pub assets_index: &'a str,
    pub extra: HashMap<&'a str, &'a str>,
}

pub fn evaluate_arguments(
    values: &[ArgumentValue],
    context: &ArgumentContext<'_>,
    features: &FeatureSet,
    platform: Platform,
) -> Vec<String> {
    let mut args = Vec::new();
    for value in values {
        match value {
            ArgumentValue::String(raw) => args.push(replace_placeholders(raw, context)),
            ArgumentValue::Ruled { rules, value } => {
                if evaluate_rules(rules, platform, features) {
                    match value {
                        StringOrVec::String(raw) => args.push(replace_placeholders(raw, context)),
                        StringOrVec::Vec(raw_values) => {
                            args.extend(raw_values.iter().map(|raw| replace_placeholders(raw, context)));
                        }
                    }
                }
            }
        }
    }
    args
}

pub fn replace_placeholders(raw: &str, context: &ArgumentContext<'_>) -> String {
    let version_name = context.version.id.as_deref().unwrap_or_default();
    let assets_root = context.minecraft_dir.join("assets");
    let library_directory = context.minecraft_dir.join("libraries");
    let game_assets = assets_root.join("virtual").join("legacy");

    let mut value = raw.to_string();
    let replacements = [
        ("${natives_directory}", context.natives_dir.to_string_lossy().to_string()),
        ("${launcher_name}", context.launcher_name.to_string()),
        ("${launcher_version}", context.launcher_version.to_string()),
        ("${classpath}", context.classpath.to_string()),
        ("${auth_player_name}", context.account.username().to_string()),
        ("${version_name}", version_name.to_string()),
        ("${game_directory}", context.game_dir.to_string_lossy().to_string()),
        ("${assets_root}", assets_root.to_string_lossy().to_string()),
        ("${assets_index_name}", context.assets_index.to_string()),
        ("${auth_uuid}", context.account.uuid().to_string()),
        ("${auth_access_token}", context.account.access_token().to_string()),
        ("${user_type}", "msa".to_string()),
        ("${version_type}", context.version_type.to_string()),
        ("${user_properties}", "{}".to_string()),
        ("${game_assets}", game_assets.to_string_lossy().to_string()),
        ("${auth_session}", context.account.access_token().to_string()),
        ("${library_directory}", library_directory.to_string_lossy().to_string()),
        ("${classpath_separator}", classpath_separator().to_string()),
    ];

    for (key, replacement) in replacements {
        value = value.replace(key, &replacement);
    }
    for (key, replacement) in &context.extra {
        value = value.replace(key, replacement);
    }
    value
}

pub fn classpath_separator() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}
```

Create `src/core/classpath.rs`:

```rust
use std::path::{Path, PathBuf};

use crate::{
    core::{maven::MavenCoordinate, version::VersionJson},
    Result,
};

pub fn classpath_entries(version: &VersionJson, minecraft_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let minecraft_dir = minecraft_dir.as_ref();
    let mut entries = Vec::new();

    for library in &version.libraries {
        let coordinate = MavenCoordinate::parse(&library.name)?;
        entries.push(minecraft_dir.join("libraries").join(coordinate.artifact_path()));
    }

    let jar_id = version.jar.as_ref().or(version.id.as_ref());
    if let Some(id) = jar_id {
        entries.push(
            minecraft_dir
                .join("versions")
                .join(id)
                .join(format!("{id}.jar")),
        );
    }

    Ok(entries)
}

pub fn classpath_string(entries: &[PathBuf]) -> String {
    let separator = super::arguments::classpath_separator();
    entries
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(separator)
}
```

Modify `src/core/mod.rs`:

```rust
pub mod arguments;
pub mod classpath;
pub mod maven;
pub mod rules;
pub mod version;
```

- [x] **Step 5: Implement launch command builder**

Create `src/command/builder.rs`:

```rust
use std::path::PathBuf;

use crate::{
    account::Account,
    core::{
        arguments::{ArgumentContext, evaluate_arguments},
        classpath::{classpath_entries, classpath_string},
        rules::FeatureSet,
        version::VersionJson,
    },
    platform::Platform,
    LauncherError, Result,
};

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub account: Account,
    pub java_executable: Option<PathBuf>,
    pub game_directory: Option<PathBuf>,
    pub natives_directory: Option<PathBuf>,
    pub launcher_name: String,
    pub launcher_version: String,
    pub custom_resolution: Option<(u32, u32)>,
    pub demo: bool,
    pub server: Option<(String, Option<u16>)>,
    pub disable_multiplayer: bool,
    pub disable_chat: bool,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            account: Account::offline("Steve"),
            java_executable: None,
            game_directory: None,
            natives_directory: None,
            launcher_name: "mc-launcher-core".to_string(),
            launcher_version: env!("CARGO_PKG_VERSION").to_string(),
            custom_resolution: None,
            demo: false,
            server: None,
            disable_multiplayer: false,
            disable_chat: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub env: Vec<(String, String)>,
}

impl LaunchCommand {
    pub fn to_process_parts(&self) -> (PathBuf, Vec<String>) {
        (self.executable.clone(), self.args.clone())
    }
}

pub fn build_launch_command(
    version: &VersionJson,
    minecraft_dir: PathBuf,
    options: LaunchOptions,
) -> Result<LaunchCommand> {
    let version_id = version.id.as_deref().ok_or_else(|| LauncherError::MissingField {
        context: "version json".to_string(),
        field: "id".to_string(),
    })?;
    let main_class = version
        .main_class
        .clone()
        .ok_or_else(|| LauncherError::MissingField {
            context: version_id.to_string(),
            field: "mainClass".to_string(),
        })?;

    let game_dir = options.game_directory.clone().unwrap_or_else(|| minecraft_dir.clone());
    let natives_dir = options
        .natives_directory
        .clone()
        .unwrap_or_else(|| minecraft_dir.join("versions").join(version_id).join("natives"));
    let entries = classpath_entries(version, &minecraft_dir)?;
    let classpath = classpath_string(&entries);
    let assets_index = version.assets.as_deref().unwrap_or(version_id);
    let version_type = version.r#type.as_deref().unwrap_or("release");

    let features = FeatureSet {
        demo_user: options.demo,
        custom_resolution: options.custom_resolution.is_some(),
        ..Default::default()
    };
    let context = ArgumentContext {
        minecraft_dir: &minecraft_dir,
        natives_dir: &natives_dir,
        game_dir: &game_dir,
        version,
        account: &options.account,
        classpath: &classpath,
        launcher_name: &options.launcher_name,
        launcher_version: &options.launcher_version,
        version_type,
        assets_index,
        extra: Default::default(),
    };

    let executable = options.java_executable.unwrap_or_else(|| PathBuf::from("java"));
    let mut args = evaluate_arguments(&version.arguments.jvm, &context, &features, Platform::current());
    args.push(main_class);

    if version.minecraft_arguments.is_some() {
        let legacy = version
            .minecraft_arguments
            .as_deref()
            .unwrap_or_default()
            .split(' ')
            .map(|part| crate::core::arguments::replace_placeholders(part, &context));
        args.extend(legacy);
    } else {
        args.extend(evaluate_arguments(
            &version.arguments.game,
            &context,
            &features,
            Platform::current(),
        ));
    }

    if let Some((width, height)) = options.custom_resolution {
        args.extend(["--width".to_string(), width.to_string(), "--height".to_string(), height.to_string()]);
    }
    if options.demo {
        args.push("--demo".to_string());
    }
    if let Some((server, port)) = options.server {
        args.extend(["--server".to_string(), server]);
        if let Some(port) = port {
            args.extend(["--port".to_string(), port.to_string()]);
        }
    }
    if options.disable_multiplayer {
        args.push("--disableMultiplayer".to_string());
    }
    if options.disable_chat {
        args.push("--disableChat".to_string());
    }

    Ok(LaunchCommand {
        executable,
        args,
        working_dir: game_dir,
        env: Vec::new(),
    })
}
```

Modify `src/command/mod.rs` to expose `builder` while retaining legacy functions until later wrapper cleanup:

```rust
pub mod builder;
```

If this removes old functions and examples fail, re-add old functions as deprecated wrappers in Task 10.

- [x] **Step 6: Run focused tests**

Run:

```bash
cargo test --test command_builder
```

Expected: PASS.

- [x] **Step 7: Commit**

```bash
git add src/account.rs src/core src/command tests/command_builder.rs
git commit -m "feat: add launch command builder"
```

## Task 5: Download Tasks And Safe Archive Extraction

**Files:**
- Create: `src/progress.rs`
- Create: `src/net/http.rs`
- Create: `src/net/download.rs`
- Modify: `src/net/mod.rs`
- Create: `src/io/archive.rs`
- Modify: `src/io/mod.rs`
- Test: `tests/download_archive.rs`

- [x] **Step 1: Write failing tests for download task skip and zip-slip rejection**

Create `tests/download_archive.rs`:

```rust
use std::{fs, io::Write};

use mc_launcher_core::{
    io::archive::extract_zip_safely,
    net::download::{Checksum, DownloadTask, should_skip_existing},
};

#[test]
fn skips_existing_file_when_sha1_matches() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("hello.txt");
    fs::write(&file, b"minecraft").unwrap();

    let task = DownloadTask {
        url: "https://example.invalid/hello.txt".to_string(),
        destination: file,
        checksum: Some(Checksum::Sha1(
            "d33f8bf35eac3e8b9f5b5b86433f62d4bda5c9f1".to_string(),
        )),
        label: "hello".to_string(),
    };

    assert!(should_skip_existing(&task).unwrap());
}

#[test]
fn rejects_zip_entry_that_escapes_destination() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("bad.zip");
    let file = fs::File::create(&zip_path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file("../escape.txt", zip::write::FileOptions::default())
        .unwrap();
    writer.write_all(b"bad").unwrap();
    writer.finish().unwrap();

    let err = extract_zip_safely(&zip_path, dir.path().join("out")).unwrap_err();
    assert!(err.to_string().contains("unsafe path"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test download_archive
```

Expected: FAIL because download and archive helpers are missing.

- [x] **Step 3: Implement progress events**

Create `src/progress.rs`:

```rust
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
    StageStarted { stage: InstallStage },
    TaskStarted { label: String, path: PathBuf },
    TaskSkipped { label: String, reason: SkipReason },
    TaskFinished { label: String },
    BytesReceived { label: String, received: u64, total: Option<u64> },
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
```

- [x] **Step 4: Implement HTTP client wrapper**

Create `src/net/http.rs`:

```rust
use reqwest::blocking::Client;

use crate::Result;

pub fn user_agent() -> String {
    format!("mc-launcher-core/{}", env!("CARGO_PKG_VERSION"))
}

pub fn client() -> Result<Client> {
    Ok(Client::builder().user_agent(user_agent()).build()?)
}

pub fn get_text(url: &str) -> Result<String> {
    Ok(client()?.get(url).send()?.error_for_status()?.text()?)
}

pub fn get_json<T>(url: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(client()?.get(url).send()?.error_for_status()?.json()?)
}
```

- [x] **Step 5: Implement download task model**

Create `src/net/download.rs`:

```rust
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

pub fn execute_plan(
    plan: &DownloadPlan,
    reporter: &mut dyn ProgressReporter,
) -> Result<()> {
    let client = super::http::client()?;
    for task in &plan.tasks {
        if should_skip_existing(task)? {
            reporter.report(ProgressEvent::TaskSkipped {
                label: task.label.clone(),
                reason: match task.checksum {
                    Some(_) => SkipReason::ChecksumMatched,
                    None => SkipReason::FileExistsWithoutChecksum,
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
```

Modify `src/net/mod.rs`:

```rust
pub mod download;
pub mod http;
```

- [x] **Step 6: Implement safe archive extraction**

Create `src/io/archive.rs`:

```rust
use std::{
    fs::{self, File},
    io,
    path::Path,
};

use zip::ZipArchive;

use crate::{
    io::paths::safe_join,
    Result,
};

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
```

Modify `src/io/mod.rs`:

```rust
pub mod archive;
pub mod hash;
pub mod paths;
```

- [x] **Step 7: Run focused tests**

Run:

```bash
cargo test --test download_archive
```

Expected: PASS.

- [x] **Step 8: Commit**

```bash
git add src/progress.rs src/net src/io tests/download_archive.rs
git commit -m "feat: add download plan and safe archive extraction"
```

## Task 6: Vanilla Install Planning And Launcher Facade

**Files:**
- Create: `src/install/request.rs`
- Create: `src/install/vanilla.rs`
- Create: `src/install/libraries.rs`
- Create: `src/install/assets.rs`
- Modify: `src/install/mod.rs`
- Create: `src/launcher.rs`
- Create: `src/prelude.rs`
- Test: `tests/install_plan.rs`

- [x] **Step 1: Add fixture and failing install plan test**

Create `tests/fixtures/version_1_20_4_min.json`:

```json
{
  "id": "1.20.4",
  "type": "release",
  "mainClass": "net.minecraft.client.main.Main",
  "minimumLauncherVersion": 21,
  "assets": "12",
  "assetIndex": {
    "id": "12",
    "sha1": "asset-index-sha1",
    "size": 1,
    "totalSize": 1,
    "url": "https://example.invalid/assets/12.json"
  },
  "downloads": {
    "client": {
      "sha1": "client-sha1",
      "size": 1,
      "url": "https://example.invalid/client.jar"
    }
  },
  "libraries": [
    {
      "name": "com.example:demo:1.0",
      "downloads": {
        "artifact": {
          "path": "com/example/demo/1.0/demo-1.0.jar",
          "sha1": "demo-sha1",
          "size": 1,
          "url": "https://example.invalid/demo-1.0.jar"
        }
      }
    }
  ],
  "arguments": {
    "jvm": ["-cp", "${classpath}"],
    "game": ["--username", "${auth_player_name}"]
  }
}
```

Create `tests/install_plan.rs`:

```rust
use mc_launcher_core::{
    core::version::VersionJson,
    install::{
        request::{InstallRequest, JavaInstallPolicy},
        vanilla::plan_vanilla_downloads,
    },
};

#[test]
fn plans_client_library_and_asset_index_downloads() {
    let version: VersionJson =
        serde_json::from_str(include_str!("fixtures/version_1_20_4_min.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let plan = plan_vanilla_downloads(&version, dir.path()).unwrap();

    let destinations = plan
        .tasks
        .iter()
        .map(|task| task.destination.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    assert!(destinations.iter().any(|path| path.ends_with("versions/1.20.4/1.20.4.jar")));
    assert!(destinations.iter().any(|path| path.ends_with("libraries/com/example/demo/1.0/demo-1.0.jar")));
    assert!(destinations.iter().any(|path| path.ends_with("assets/indexes/12.json")));
}

#[test]
fn install_request_defaults_to_auto_java() {
    let request = InstallRequest::vanilla("1.20.4");
    assert_eq!(request.minecraft_version, "1.20.4");
    assert_eq!(request.java, JavaInstallPolicy::Auto);
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test install_plan
```

Expected: FAIL because install request and vanilla planning are missing.

- [x] **Step 3: Implement install request types**

Create `src/install/request.rs`:

```rust
use crate::loader::common::LoaderSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    pub minecraft_version: String,
    pub loader: Option<LoaderSpec>,
    pub java: JavaInstallPolicy,
}

impl InstallRequest {
    pub fn vanilla(version: impl Into<String>) -> Self {
        Self {
            minecraft_version: version.into(),
            loader: None,
            java: JavaInstallPolicy::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaInstallPolicy {
    Auto,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    pub version_id: String,
}
```

Create `src/loader/common.rs`:

```rust
use crate::loader::LoaderKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderVersion {
    Latest,
    LatestStable,
    Exact(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderSpec {
    Fabric { version: LoaderVersion },
    Quilt { version: LoaderVersion },
    Forge { version: LoaderVersion },
    NeoForge { version: LoaderVersion },
}

impl LoaderSpec {
    pub fn kind(&self) -> LoaderKind {
        match self {
            Self::Fabric { .. } => LoaderKind::Fabric,
            Self::Quilt { .. } => LoaderKind::Quilt,
            Self::Forge { .. } => LoaderKind::Forge,
            Self::NeoForge { .. } => LoaderKind::NeoForge,
        }
    }
}
```

Modify `src/loader/mod.rs`:

```rust
pub mod common;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}
```

- [x] **Step 4: Implement vanilla planning**

Create `src/install/libraries.rs`:

```rust
use std::path::Path;

use crate::{
    core::version::Library,
    net::download::{Checksum, DownloadTask},
    Result,
};

pub fn plan_library_downloads(libraries: &[Library], minecraft_dir: &Path) -> Result<Vec<DownloadTask>> {
    let mut tasks = Vec::new();
    for library in libraries {
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                tasks.push(DownloadTask {
                    url: artifact.url.clone(),
                    destination: minecraft_dir.join("libraries").join(&artifact.path),
                    checksum: Some(Checksum::Sha1(artifact.sha1.clone())),
                    label: library.name.clone(),
                });
            }
        }
    }
    Ok(tasks)
}
```

Create `src/install/assets.rs`:

```rust
use std::path::Path;

use crate::{
    core::version::VersionJson,
    net::download::{Checksum, DownloadTask},
    Result,
};

pub fn plan_asset_index_download(version: &VersionJson, minecraft_dir: &Path) -> Result<Vec<DownloadTask>> {
    let Some(asset_index) = &version.asset_index else {
        return Ok(Vec::new());
    };
    Ok(vec![DownloadTask {
        url: asset_index.url.clone(),
        destination: minecraft_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", asset_index.id)),
        checksum: Some(Checksum::Sha1(asset_index.sha1.clone())),
        label: format!("assets index {}", asset_index.id),
    }])
}
```

Create `src/install/vanilla.rs`:

```rust
use std::path::Path;

use crate::{
    core::version::VersionJson,
    net::download::{Checksum, DownloadPlan, DownloadTask},
    LauncherError, Result,
};

pub fn plan_vanilla_downloads(version: &VersionJson, minecraft_dir: impl AsRef<Path>) -> Result<DownloadPlan> {
    let minecraft_dir = minecraft_dir.as_ref();
    let version_id = version.id.as_deref().ok_or_else(|| LauncherError::MissingField {
        context: "version json".to_string(),
        field: "id".to_string(),
    })?;

    let mut plan = DownloadPlan::default();
    if let Some(client) = version.downloads.get("client") {
        plan.tasks.push(DownloadTask {
            url: client.url.clone(),
            destination: minecraft_dir
                .join("versions")
                .join(version_id)
                .join(format!("{version_id}.jar")),
            checksum: Some(Checksum::Sha1(client.sha1.clone())),
            label: format!("client {version_id}"),
        });
    }

    plan.tasks.extend(super::libraries::plan_library_downloads(&version.libraries, minecraft_dir)?);
    plan.tasks.extend(super::assets::plan_asset_index_download(version, minecraft_dir)?);
    Ok(plan)
}
```

Modify `src/install/mod.rs`:

```rust
pub mod assets;
pub mod libraries;
pub mod request;
pub mod vanilla;

pub use request::{InstallRequest, InstallResult, JavaInstallPolicy};
```

- [x] **Step 5: Add facade and prelude skeleton**

Create `src/launcher.rs`:

```rust
use std::path::{Path, PathBuf};

use crate::{
    command::builder::{LaunchCommand, LaunchOptions, build_launch_command},
    core::version::VersionJson,
    install::request::{InstallRequest, InstallResult},
    Result,
};

#[derive(Debug, Clone)]
pub struct Launcher {
    minecraft_dir: PathBuf,
}

impl Launcher {
    pub fn new(minecraft_dir: impl Into<PathBuf>) -> Self {
        Self {
            minecraft_dir: minecraft_dir.into(),
        }
    }

    pub fn minecraft_dir(&self) -> &Path {
        &self.minecraft_dir
    }

    pub fn install(&self, request: InstallRequest) -> Result<InstallResult> {
        Ok(InstallResult {
            version_id: request.minecraft_version,
        })
    }

    pub fn build_launch_command_from_version(
        &self,
        version: &VersionJson,
        options: LaunchOptions,
    ) -> Result<LaunchCommand> {
        build_launch_command(version, self.minecraft_dir.clone(), options)
    }
}
```

Create `src/prelude.rs`:

```rust
pub use crate::{
    account::Account,
    command::builder::{LaunchCommand, LaunchOptions},
    error::{LauncherError, Result},
    install::request::{InstallRequest, InstallResult, JavaInstallPolicy},
    launcher::Launcher,
    loader::{
        LoaderKind,
        common::{LoaderSpec, LoaderVersion},
    },
    progress::{ProgressEvent, ProgressReporter},
};
```

- [x] **Step 6: Run focused tests**

Run:

```bash
cargo test --test install_plan
```

Expected: PASS.

- [x] **Step 7: Commit**

```bash
git add src/install src/launcher.rs src/prelude.rs src/loader tests/fixtures/version_1_20_4_min.json tests/install_plan.rs
git commit -m "feat: add install request and vanilla download planning"
```
