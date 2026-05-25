# Launcher Core Redesign

Date: 2026-05-25

## Summary

This project is a Rust library for building Minecraft Java Edition launchers. The current code provides basic vanilla install, offline/Microsoft authentication, command generation, Java runtime install, and a thin Forge helper. It was written early in the project and now has several maintainability issues:

- public APIs are mostly loose functions with unclear ownership boundaries;
- install, download, path validation, JSON merging, and progress reporting are coupled;
- errors are mostly `Box<dyn Error>` or string errors;
- many code paths rely on `unwrap` and panic-prone assumptions;
- dependencies include outdated or platform-sensitive crates, including unconditional `winver`, which currently makes `cargo test` fail on macOS;
- Fabric, Quilt, NeoForge, and MRPack support is absent or only represented by placeholder types.

The first redesign phase will introduce a maintainable core architecture and add first-class support for Fabric, Quilt, Forge, and NeoForge. MRPack support is intentionally deferred to a later phase because it adds modpack file selection, overrides, dependency projects, and index semantics beyond the core launch pipeline.

## Goals

- Reorganize the library around stable domain boundaries rather than legacy file layout.
- Allow breaking API changes before a future `0.1.0` release.
- Add loader installation and launch support for Fabric, Quilt, Forge, and NeoForge.
- Keep the primary SDK usable from synchronous Rust callers in this phase.
- Make the default test suite work without network access.
- Fix cross-platform compilation problems, especially unconditional Windows-only dependencies.
- Replace panic-prone behavior in main flows with typed errors.
- Preserve migration paths for the listed important old functions through deprecated wrappers.

## Non-Goals

- Do not implement MRPack in this phase.
- Do not make the public API async-first in this phase.
- Do not build a GUI, CLI, or complete launcher application.
- Do not guarantee compatibility for every legacy public type path if it blocks the redesign.
- Do not run Minecraft as part of automated tests.

## Current Baseline

The current project layout is:

- `auth`: Microsoft and offline account helpers.
- `command`: launch command generation.
- `install`: vanilla install orchestration.
- `runtime`: Mojang Java runtime install and discovery.
- `forge`: basic Forge metadata and installer helpers.
- `utils`: networking, caching, rule parsing, natives, Java discovery, and miscellaneous helpers.
- `types`: mixed public and internal data structures.

The current `cargo test` baseline fails on macOS after dependencies download because `winver` is compiled unconditionally and emits a compile error on non-Windows targets. This is a first-order fix in the redesign.

## Sources And Protocol Assumptions

The design is based on the current codebase and these launcher metadata sources:

- Mojang version manifest and version JSON, as described by launcher tutorials and used by the existing project.
- Fabric Meta API, including loader lists and `profile/json` endpoints.
- Quilt Meta API, including loader lists and `profile/json` endpoints.
- Forge Maven metadata plus installer-generated client profiles.
- NeoForge Maven metadata plus installer-generated client profiles.

Fabric and Quilt are treated as metadata-profile loaders because their meta APIs return launcher-compatible profile JSON that inherits from a vanilla version. Forge and NeoForge are treated as installer loaders because their supported path is to run an installer JAR that writes or prepares the launcher version profile and libraries.

## Architecture

The first phase will reorganize the crate into four main layers.

### `core`

`core` owns pure launcher domain behavior:

- Minecraft version JSON models.
- Version inheritance and merge semantics.
- Maven coordinate parsing and artifact path generation.
- OS, architecture, and feature rule evaluation.
- Argument placeholder replacement.
- Classpath construction rules.
- Resolved version models used by install and command generation.

This layer should avoid networking and filesystem side effects where possible. Most logic here should be covered by unit tests using fixtures.

### `net` And `io`

`net` and `io` own external effects:

- HTTP client configuration and user agent.
- Small metadata fetches with cache policy.
- File downloads.
- Checksum verification.
- Zip and native extraction.
- LZMA or compressed runtime handling.
- Canonical path validation.
- Safe creation of directories and files.

The main install path should use `DownloadTask` values rather than making ad hoc download calls. This makes progress reporting, retry behavior, checksum behavior, and cache skips consistent.

### `install`

`install` owns orchestration:

- Resolve an `InstallRequest`.
- Fetch or load the target vanilla version.
- Ask a loader provider for a loader profile or installer output when requested.
- Merge version inheritance into a `ResolvedVersion`.
- Generate a download plan for client JAR, libraries, assets, logging config, natives, and Java runtime.
- Execute download tasks.
- Extract natives and validate installed outputs.

`install` does not contain Fabric, Quilt, Forge, or NeoForge-specific logic beyond consuming a loader provider interface.

### `loader`

`loader` owns mod loader adapters:

- `FabricProvider`
- `QuiltProvider`
- `ForgeProvider`
- `NeoForgeProvider`

Each provider exposes a common capability shape:

- list supported Minecraft versions when the upstream source provides it;
- list loader versions;
- resolve latest or latest stable loader versions;
- install or generate a launcher version profile for a requested Minecraft version;
- return the installed version ID.

Fabric and Quilt providers fetch profile JSON from meta APIs and write it under `versions/<id>/<id>.json`.

Forge and NeoForge providers list versions from Maven metadata, download installer JARs, run client installation with a selected Java executable, then read the generated version JSON. The core install pipeline then validates and completes required assets, libraries, natives, and runtime.

## Public API

The new primary API will use a facade:

```rust
use mc_launcher_core::prelude::*;

let launcher = Launcher::new(minecraft_dir);

let install = launcher.install(InstallRequest {
    minecraft_version: "1.20.4".into(),
    loader: Some(LoaderSpec::Fabric {
        version: LoaderVersion::LatestStable,
    }),
    java: JavaInstallPolicy::Auto,
    ..Default::default()
})?;

let command = launcher.build_launch_command(
    &install.version_id,
    LaunchOptions {
        account: Account::offline("Steve"),
        ..Default::default()
    },
)?;
```

The `prelude` should expose the main user-facing types:

- `Launcher`
- `InstallRequest`
- `InstallResult`
- `LoaderSpec`
- `LoaderKind`
- `LoaderVersion`
- `JavaInstallPolicy`
- `LaunchOptions`
- `LaunchCommand`
- `Account`
- `ProgressEvent`
- `LauncherError`

Deprecated wrappers will remain for these common old entry points:

- `install::install_minecraft_version`
- `command::get_minecraft_command`
- `utils::get_latest_version`
- `utils::get_version_list`
- `auth::offline::get_offline_options`

Wrappers will call the new implementation. Legacy public functions outside this list may be removed or replaced with documented equivalents when keeping them would preserve the old architecture.

## Data Flow

### Vanilla Install

1. Read `InstallRequest`.
2. Resolve the vanilla version from local files or Mojang manifest.
3. Write or refresh the version JSON when needed.
4. Merge inherited versions into `ResolvedVersion`.
5. Build a `DownloadPlan`.
6. Download and verify libraries, client JAR, assets index, asset objects, logging config, and Java runtime if requested.
7. Extract natives.
8. Return `InstallResult`.

### Fabric And Quilt Install

1. Resolve requested loader version.
2. Fetch the loader profile JSON from the provider meta API.
3. Write the loader profile to `versions/<loader-id>/<loader-id>.json`.
4. Install the inherited vanilla version.
5. Merge the loader profile with vanilla version data.
6. Generate and execute download tasks for loader libraries plus vanilla requirements.
7. Return the loader version ID.

### Forge And NeoForge Install

1. Resolve requested loader version from Maven metadata.
2. Download the installer JAR using checksum when available.
3. Run installer in client mode using selected Java.
4. Read the generated launcher version profile.
5. Install or complete the inherited vanilla version.
6. Merge the generated profile with vanilla version data.
7. Generate and execute any remaining download tasks.
8. Return the installed loader version ID.

Installer execution must be isolated to a temporary file path and must clean up temporary files after success or failure when possible.

## Error Handling

Introduce a typed error enum using `thiserror`:

```rust
pub enum LauncherError {
    Network { source: reqwest::Error },
    Io { source: std::io::Error },
    Json { source: serde_json::Error },
    InvalidVersionId { id: String },
    UnsupportedPlatform { os: String, arch: String },
    ChecksumMismatch { path: PathBuf, expected: String, actual: String },
    UnsafePath { base: PathBuf, path: PathBuf },
    InvalidMavenCoordinate { coordinate: String },
    LoaderVersionNotFound { loader: LoaderKind, version: String },
    InstallerFailed { loader: LoaderKind, status: Option<i32> },
    MissingField { context: String, field: String },
}
```

Implementation may add more specific variants, but install and command generation paths must not depend on `unwrap`, `panic!`, or stringly typed errors for expected failure cases.

## Dependency Strategy

- Upgrade `reqwest` to a current maintained version while keeping blocking support for this phase.
- Replace `rust-crypto` with the maintained `sha1` crate already present transitively.
- Replace direct `ring` usage in auth PKCE hashing with the maintained RustCrypto digest stack unless another direct `ring` use remains necessary.
- Move `winver` into target-specific Windows dependencies if Windows version detection remains; otherwise remove it.
- Upgrade `zip` and use enclosed-path extraction APIs where available, backed by explicit zip-slip checks for every archive write.
- Replace `lazy_static` with `once_cell`.
- Keep Tokio out of the public API unless required by updated dependencies.

## Progress Reporting

Replace `CallbackDict` as the primary progress API with structured progress events:

```rust
pub enum ProgressEvent {
    StageStarted { stage: InstallStage },
    TaskStarted { label: String, path: PathBuf },
    TaskSkipped { label: String, reason: SkipReason },
    TaskFinished { label: String },
    BytesReceived { label: String, received: u64, total: Option<u64> },
}
```

The synchronous facade can accept an optional callback or reporter trait. Deprecated wrappers can adapt old `CallbackDict` callbacks into progress events.

## Path Safety

Path safety must be enforced before writing files:

- Resolve target paths relative to the Minecraft directory.
- Canonicalize the base directory.
- Canonicalize existing parents, then validate that the resulting path remains inside the base directory.
- Reject archive entries that would escape the target directory.
- Avoid trusting paths from remote JSON without validation.

This replaces the current `starts_with` check, which can be insufficient without canonicalization.

## Command Generation

Command generation should consume a `ResolvedVersion` and `LaunchOptions`.

Responsibilities:

- choose Java executable from explicit option, installed Mojang runtime, or system fallback;
- build classpath from resolved libraries and client JAR;
- apply rule-filtered JVM and game arguments;
- replace known placeholders;
- include optional server, resolution, demo, logging, quick play, multiplayer, and chat flags;
- return `LaunchCommand` with executable, args, working directory, and optional environment values.

Unknown placeholders should either remain explicit in a diagnostic mode or produce a typed error in strict mode. The default should prefer compatibility with official launcher JSON while avoiding silent panics.

## Testing

Default tests should not require network access.

Required test groups:

- Maven coordinate parsing and artifact path generation.
- Rule evaluation for OS, architecture, and feature flags.
- Version inheritance merge behavior.
- Placeholder replacement and command generation from fixtures.
- Download task planning without executing network downloads.
- Fabric and Quilt profile parsing from checked-in fixtures.
- Forge and NeoForge version ID parsing and Maven metadata parsing from checked-in fixtures.
- Path traversal rejection for downloads and zip extraction.
- Cross-platform compilation checks where possible.

Network smoke tests may be marked ignored or feature-gated. They can verify live Mojang, Fabric, Quilt, Forge, and NeoForge metadata when explicitly requested.

## Migration Plan

1. Add the new modules and typed errors.
2. Move pure JSON, Maven, rule, and argument behavior into `core` with tests.
3. Introduce `DownloadTask` and path-safe IO helpers.
4. Rebuild vanilla install on the new plan.
5. Rebuild command generation around `ResolvedVersion`.
6. Add Fabric and Quilt providers.
7. Add Forge and NeoForge providers.
8. Add deprecated wrappers for important old functions.
9. Update README examples to use the new facade.
10. Remove or hide obsolete placeholder types that are not part of the new API.

## Open Decisions Resolved

- Breaking API changes are allowed.
- The first loader phase includes Fabric, Quilt, Forge, and NeoForge.
- MRPack is deferred.
- The recommended architecture is layered core plus loader providers.
- The first phase keeps a synchronous public API.

## Acceptance Criteria

- `cargo test` passes on macOS after target-specific dependency fixes.
- The default test suite does not require network access.
- Vanilla install and command generation work through the new `Launcher` facade.
- Fabric and Quilt loader profiles can be installed through the new provider model.
- Forge and NeoForge versions can be listed and installed through the installer provider model.
- Old common entry points either work through wrappers or have clear documented replacements.
- Main install and command-generation paths return `LauncherError` instead of panicking for expected errors.
- README shows the new primary API.
