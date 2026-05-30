# Launcher Core Redesign Phase 4: Compatibility, Docs, And Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore documented legacy wrappers, migrate auth hashing, update examples, and complete verification.

**Architecture:** This phase makes the redesign usable for existing callers while documenting the new facade. It ends with formatting, offline tests, example compilation, dependency residue checks, and optional live metadata smoke tests.

**Tech Stack:** Rust 2021, `reqwest` blocking client, `serde`, `serde_json`, `thiserror`, `once_cell`, RustCrypto `sha1`/`sha2`/`digest`, `zip`, `tempfile`, Cargo integration tests.

---

## Phase File Map

- Modify `src/auth/offline.rs` to bridge old offline options to the new account model.
- Modify `src/auth/microsoft_account.rs` to replace direct `ring` digest usage.
- Modify `src/utils/mod.rs`, `src/utils/helper.rs`, `src/command/mod.rs`, `src/install/mod.rs`, and `src/types/mod.rs` for compatibility wrappers.
- Modify `README.md`, `examples/simple_launch.rs`, and `examples/simple_offline_launch.rs` for the new facade.
- Add `tests/compat_wrappers.rs` and `tests/live_metadata.rs`.
- Run final verification across formatting, tests, examples, dependency residue, and panic-prone path checks.

## Task 10: Compatibility Wrappers And Auth Digest Migration

**Files:**
- Modify: `src/auth/offline.rs`
- Modify: `src/auth/microsoft_account.rs`
- Modify: `src/utils/helper.rs`
- Modify: `src/utils/mod.rs`
- Modify: `src/command/mod.rs`
- Modify: `src/install/mod.rs`
- Modify: `src/types/mod.rs`
- Test: `tests/compat_wrappers.rs`

- [x] **Step 1: Write failing compatibility wrapper tests**

Create `tests/compat_wrappers.rs`:

```rust
use mc_launcher_core::{
    auth::offline::get_offline_options,
    command::get_minecraft_command,
    install::install_minecraft_version,
    utils::get_core_version,
};

#[test]
fn offline_options_still_returns_username() {
    let options = get_offline_options("Steve");
    assert_eq!(options.username.as_deref(), Some("Steve"));
    assert!(options.uuid.is_some());
}

#[test]
fn core_version_is_package_version() {
    assert_eq!(get_core_version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn missing_version_command_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = get_minecraft_command("missing", dir.path(), &Default::default()).unwrap_err();
    assert!(err.to_string().contains("missing"));
}

#[test]
fn vanilla_install_wrapper_accepts_request_shape() {
    let dir = tempfile::tempdir().unwrap();
    let result = install_minecraft_version("1.20.4", dir.path(), &Default::default());
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("network"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test compat_wrappers
```

Expected: FAIL where wrappers were removed or now point at old dependency APIs.

- [x] **Step 3: Keep legacy offline helper using new account model**

Modify `src/auth/offline.rs`:

```rust
use crate::{account::Account, types::MinecraftOptions};

pub fn get_offline_account(user_name: &str) -> Account {
    Account::offline(user_name)
}

#[deprecated(note = "use Account::offline")]
pub fn get_offline_options(user_name: &str) -> MinecraftOptions {
    let account = Account::offline(user_name);
    MinecraftOptions::new(
        account.username().to_string(),
        account.uuid().to_string(),
        account.access_token().to_string(),
    )
}
```

- [x] **Step 4: Replace PKCE hashing dependency**

Modify `src/auth/microsoft_account.rs`:

```rust
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distr::Alphanumeric, Rng};
use reqwest::blocking::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use url::Url;
```

Replace the PKCE digest lines with:

```rust
let digest = Sha256::digest(code_verifier.as_bytes());
let code_challenge = URL_SAFE_NO_PAD.encode(digest);
```

Apply the same replacement in tests inside this module.

- [x] **Step 5: Re-add command and install wrappers**

In `src/command/mod.rs`, expose the builder and a legacy wrapper:

```rust
pub mod builder;

use std::{fs, path::Path};

use crate::{core::version::VersionJson, types::MinecraftOptions, LauncherError};

#[deprecated(note = "use Launcher::build_launch_command_from_version")]
pub fn get_minecraft_command(
    version: &str,
    minecraft_directory: impl AsRef<Path>,
    _options_arg: &MinecraftOptions,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = minecraft_directory
        .as_ref()
        .join("versions")
        .join(version)
        .join(format!("{version}.json"));
    if !path.is_file() {
        return Err(Box::new(LauncherError::InvalidVersionId {
            id: version.to_string(),
        }));
    }
    let version_json: VersionJson = serde_json::from_str(&fs::read_to_string(path)?)?;
    let command = builder::build_launch_command(
        &version_json,
        minecraft_directory.as_ref().to_path_buf(),
        Default::default(),
    )?;
    let mut parts = vec![command.executable.to_string_lossy().to_string()];
    parts.extend(command.args);
    Ok(parts)
}
```

In `src/install/mod.rs`, expose modules and a legacy wrapper:

```rust
pub mod assets;
pub mod libraries;
pub mod loader;
pub mod request;
pub mod vanilla;

use std::path::Path;

pub use request::{InstallRequest, InstallResult, JavaInstallPolicy};

#[deprecated(note = "use Launcher::install")]
pub fn install_minecraft_version(
    version_id: &str,
    minecraft_directory: impl AsRef<Path>,
    _callback: &crate::types::CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    let launcher = crate::launcher::Launcher::new(minecraft_directory.as_ref().to_path_buf());
    launcher.install(InstallRequest::vanilla(version_id))?;
    Ok(())
}
```

- [x] **Step 6: Keep utility version helpers compiling**

Modify `src/utils/mod.rs` so `get_core_version` remains:

```rust
pub fn get_core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

For old network helpers retained in `utils`, delegate to `net::http` or mark deprecated. Keep `MinecraftOptions`, `CallbackDict`, and related legacy structs in `src/types/mod.rs` until all examples and wrappers compile.

- [x] **Step 7: Run focused tests**

Run:

```bash
cargo test --test compat_wrappers
```

Expected: PASS.

- [x] **Step 8: Run auth tests**

Run:

```bash
cargo test auth::microsoft_account::test_code_challenge
```

Expected: PASS.

- [x] **Step 9: Commit**

```bash
git add src/auth src/utils src/command src/install src/types tests/compat_wrappers.rs
git commit -m "feat: add compatibility wrappers"
```

## Task 11: README, Examples, And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `examples/simple_launch.rs`
- Modify: `examples/simple_offline_launch.rs`
- Test: full suite

- [x] **Step 1: Update README example**

Replace the README example with this facade-based example:

```rust
use std::process::Command;

use mc_launcher_core::prelude::*;

fn main() -> mc_launcher_core::Result<()> {
    let minecraft_dir = mc_launcher_core::utils::get_minecraft_directory();
    let launcher = Launcher::new(minecraft_dir);

    let install = launcher.install(InstallRequest {
        minecraft_version: "1.20.4".to_string(),
        loader: Some(LoaderSpec::Fabric {
            version: LoaderVersion::LatestStable,
        }),
        java: JavaInstallPolicy::Auto,
    })?;

    let version_json_path = launcher
        .minecraft_dir()
        .join("versions")
        .join(&install.version_id)
        .join(format!("{}.json", install.version_id));
    let version_json: mc_launcher_core::core::version::VersionJson =
        serde_json::from_str(&std::fs::read_to_string(version_json_path)?)?;

    let command = launcher.build_launch_command_from_version(
        &version_json,
        LaunchOptions {
            account: Account::offline("Steve"),
            ..Default::default()
        },
    )?;

    let mut child = Command::new(command.executable).args(command.args).spawn()?;
    child.wait()?;
    Ok(())
}
```

- [x] **Step 2: Update examples**

Update `examples/simple_offline_launch.rs` to use `Launcher`, `InstallRequest::vanilla`, `Account::offline`, and `build_launch_command_from_version`.

Update `examples/simple_launch.rs` to show a Fabric install using `LoaderSpec::Fabric { version: LoaderVersion::LatestStable }`.

- [x] **Step 3: Run formatting**

Run:

```bash
cargo fmt
```

Expected: command exits successfully.

- [x] **Step 4: Run full tests**

Run:

```bash
cargo test
```

Expected: PASS. Default tests must not require live network access.

- [x] **Step 5: Run examples compile check**

Run:

```bash
cargo test --examples
```

Expected: PASS.

- [x] **Step 6: Check for legacy dependency residue**

Run:

```bash
rg -n "rust-crypto|lazy_static|ring::digest|winver" Cargo.toml src
```

Expected: no `rust-crypto`, `lazy_static`, or `ring::digest`; `winver` appears only in Windows-targeted dependency usage or not at all.

- [x] **Step 7: Check for panic-prone core paths**

Run:

```bash
rg -n "unwrap\\(|expect\\(|panic!" src/core src/install src/loader src/net src/io src/command
```

Expected: no `unwrap`, `expect`, or `panic!` in main implementation modules. Test modules may use them.

- [x] **Step 8: Commit**

```bash
git add README.md examples Cargo.toml Cargo.lock src tests
git commit -m "docs: update launcher facade examples"
```

## Task 12: Optional Live Metadata Smoke Tests

**Files:**
- Create: `tests/live_metadata.rs`

- [x] **Step 1: Add ignored live metadata tests**

Create `tests/live_metadata.rs`:

```rust
use mc_launcher_core::loader::{
    fabric, forge, neoforge, quilt,
};

#[test]
#[ignore = "requires network"]
fn fetches_live_fabric_loader_versions() {
    assert!(!fabric::list_loader_versions().unwrap().is_empty());
}

#[test]
#[ignore = "requires network"]
fn fetches_live_quilt_loader_versions() {
    assert!(!quilt::list_loader_versions().unwrap().is_empty());
}

#[test]
#[ignore = "requires network"]
fn fetches_live_forge_versions() {
    assert!(!forge::list_forge_versions().unwrap().is_empty());
}

#[test]
#[ignore = "requires network"]
fn fetches_live_neoforge_versions() {
    assert!(!neoforge::list_neoforge_versions().unwrap().is_empty());
}
```

- [x] **Step 2: Run default suite and verify ignored tests do not run**

Run:

```bash
cargo test --test live_metadata
```

Expected: PASS with four ignored tests.

- [x] **Step 3: Run live smoke tests manually**

Run:

```bash
cargo test --test live_metadata -- --ignored
```

Expected: PASS when network is available.

- [x] **Step 4: Commit**

```bash
git add tests/live_metadata.rs
git commit -m "test: add ignored live loader metadata smoke tests"
```

## Final Verification

- [x] Run:

```bash
cargo fmt -- --check
cargo test
cargo test --examples
cargo test --test live_metadata -- --ignored
```

Expected: all commands pass on the current machine with network available for the ignored live metadata suite.

- [x] Run:

```bash
git status --short
```

Expected: clean working tree after the final commit.
