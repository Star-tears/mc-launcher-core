//! Building blocks for a Rust Minecraft launcher.
//!
//! `mc-launcher-core` focuses on the parts a launcher backend needs before it
//! can hand control to Java:
//!
//! - resolving and installing vanilla, Fabric, Quilt, Forge, and NeoForge
//!   profiles;
//! - downloading client jars, libraries, assets, and native libraries;
//! - merging inherited version metadata into a launchable [`core::version::VersionJson`];
//! - building a [`command::builder::LaunchCommand`] that can be passed to
//!   [`std::process::Command`];
//! - applying compatibility metadata for older Minecraft versions on macOS
//!   Apple Silicon.
//!
//! The easiest entry point is [`launcher::Launcher`]. Most applications should
//! import [`prelude`] and keep lower-level modules for custom install or
//! inspection workflows.
//!
//! # Quick Start
//!
//! Install Fabric, load the resulting profile, build a launch command, and run
//! it with an offline account:
//!
//! ```no_run
//! use std::process::Command;
//!
//! use mc_launcher_core::prelude::*;
//!
//! fn main() -> mc_launcher_core::Result<()> {
//!     let minecraft_dir = std::env::current_dir()?.join(".minecraft");
//!     let launcher = Launcher::new(minecraft_dir);
//!
//!     let install = launcher.install(InstallRequest {
//!         minecraft_version: "1.20.1".to_string(),
//!         loader: Some(LoaderSpec::Fabric {
//!             version: LoaderVersion::LatestStable,
//!         }),
//!         java: JavaInstallPolicy::Auto,
//!     })?;
//!     let version = launcher.load_version(&install.version_id)?;
//!
//!     let command = launcher.build_launch_command_from_version(
//!         &version,
//!         LaunchOptions {
//!             account: Account::offline("Steve"),
//!             ..Default::default()
//!         },
//!     )?;
//!
//!     let mut child = Command::new(&command.executable)
//!         .args(&command.args)
//!         .current_dir(&command.working_dir)
//!         .spawn()?;
//!     child.wait()?;
//!     Ok(())
//! }
//! ```
//!
//! # Module Map
//!
//! - [`prelude`] re-exports the stable facade types for launcher applications.
//! - [`launcher`] contains the high-level install and command-building facade.
//! - [`install`] plans and executes client, asset, library, loader, and native
//!   installation work.
//! - [`command`] turns version metadata and launch options into Java process
//!   arguments.
//! - [`compatibility`] adjusts metadata for known platform gaps such as legacy
//!   macOS arm64 LWJGL support.
//! - [`auth`] contains offline and Microsoft account helpers.
//! - [`core`], [`io`], and [`net`] hold lower-level primitives used by the
//!   facade.
//!
//! # Java Runtime
//!
//! The crate does not currently bundle or manage a production Java runtime for
//! the new facade. Use [`command::builder::LaunchOptions::java_executable`] to
//! point at the runtime your launcher selected. Older compatibility wrappers in
//! [`runtime`] are retained for existing callers.
//!
//! # Error Handling
//!
//! New facade APIs return [`Result`], an alias over [`LauncherError`]. Errors
//! preserve their source where possible, so callers can display simple messages
//! or inspect variants for recovery.

#![warn(rustdoc::broken_intra_doc_links)]

pub mod account;
pub mod auth;
pub mod command;
pub mod compatibility;
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
pub mod runtime;
pub mod types;
pub mod utils;

pub use error::{LauncherError, Result};
