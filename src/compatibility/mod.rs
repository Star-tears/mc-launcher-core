//! Platform compatibility adjustments for Minecraft metadata.
//!
//! Mojang's historical version metadata does not always contain libraries that
//! work on every modern platform. This module patches selected known cases
//! before download planning or launch-command construction.

use std::collections::HashMap;

use crate::{
    core::{
        maven::MavenCoordinate,
        version::{Library, LibraryArtifact, LibraryDownloads, VersionJson},
    },
    platform::{Arch, Os, Platform},
};

/// Whether compatibility patches should be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityPolicy {
    /// Apply known compatibility patches for the target platform.
    Auto,
    /// Leave version metadata unchanged.
    Disabled,
}

/// Compatibility patch that was applied to version metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityPatch {
    /// Legacy LWJGL 2 metadata was patched for macOS arm64.
    LegacyMacArm64Lwjgl2,
    /// Older LWJGL 3 metadata was patched for macOS arm64 natives.
    MacArm64Lwjgl3,
}

/// Describes how a launcher should host the game process for window creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowingStrategy {
    /// Start Java from the launcher's current process model.
    CurrentProcess,
    /// Host Java from a real macOS app bundle or equivalent GUI app process.
    MacOsAppBundle,
}

/// Window-hosting guidance discovered while applying compatibility rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowingHint {
    /// Recommended process-hosting strategy.
    pub strategy: WindowingStrategy,
    /// Whether the caller should verify that a visible game window was created.
    pub requires_visible_window_verification: bool,
    /// Human-readable explanation for the recommendation.
    pub reason: &'static str,
}

/// Java runtime recommendation discovered while applying compatibility rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaRuntimeHint {
    /// Suggested Java major version.
    pub major_version: i32,
    /// Suggested runtime architecture.
    pub arch: Arch,
    /// Short distribution hint suitable for logs or UI.
    pub distribution_hint: &'static str,
    /// Human-readable explanation for the recommendation.
    pub reason: &'static str,
}

/// Version metadata plus compatibility guidance.
#[derive(Debug, Clone, PartialEq)]
pub struct CompatibilityResult {
    /// Possibly patched version metadata.
    pub version: VersionJson,
    /// Patches applied to produce [`CompatibilityResult::version`].
    pub applied_patches: Vec<CompatibilityPatch>,
    /// Optional Java runtime recommendation.
    pub java_runtime: Option<JavaRuntimeHint>,
    /// Window-hosting recommendation for the caller.
    pub windowing: WindowingHint,
}

/// Applies known compatibility adjustments for the given platform and policy.
///
/// The returned version is cloned from the input when no patch is needed.
pub fn apply_compatibility(
    version: &VersionJson,
    platform: Platform,
    policy: CompatibilityPolicy,
) -> CompatibilityResult {
    if policy == CompatibilityPolicy::Disabled {
        return CompatibilityResult {
            version: version.clone(),
            applied_patches: Vec::new(),
            java_runtime: None,
            windowing: current_process_windowing_hint(),
        };
    }

    if needs_legacy_macos_lwjgl2_patch(version, platform) {
        return CompatibilityResult {
            version: apply_legacy_macos_lwjgl2_patch(version),
            applied_patches: vec![CompatibilityPatch::LegacyMacArm64Lwjgl2],
            java_runtime: Some(JavaRuntimeHint {
                major_version: 8,
                arch: Arch::Aarch64,
                distribution_hint: "Azul Zulu Java 8 arm64",
                reason: "Legacy LWJGL 2 Minecraft versions need an arm64 Java 8 runtime on Apple Silicon.",
            }),
            windowing: legacy_macos_lwjgl2_windowing_hint(),
        };
    }

    if needs_macos_arm64_lwjgl3_patch(version, platform) {
        return CompatibilityResult {
            version: apply_macos_arm64_lwjgl3_patch(version),
            applied_patches: vec![CompatibilityPatch::MacArm64Lwjgl3],
            java_runtime: Some(JavaRuntimeHint {
                major_version: version
                    .java_version
                    .as_ref()
                    .map(|java| java.major_version)
                    .unwrap_or(8),
                arch: Arch::Aarch64,
                distribution_hint: "arm64 Java runtime matching version.json javaVersion",
                reason: "Older LWJGL 3 Minecraft versions need arm64 macOS native libraries on Apple Silicon.",
            }),
            windowing: current_process_windowing_hint(),
        };
    }

    CompatibilityResult {
        version: version.clone(),
        applied_patches: Vec::new(),
        java_runtime: None,
        windowing: current_process_windowing_hint(),
    }
}

fn current_process_windowing_hint() -> WindowingHint {
    WindowingHint {
        strategy: WindowingStrategy::CurrentProcess,
        requires_visible_window_verification: false,
        reason: "The version can be launched as a normal Java process by the launcher.",
    }
}

fn legacy_macos_lwjgl2_windowing_hint() -> WindowingHint {
    WindowingHint {
        strategy: WindowingStrategy::MacOsAppBundle,
        requires_visible_window_verification: true,
        reason: "Legacy LWJGL 2 can create 0x0 invisible windows when spawned directly from a CLI process on Apple Silicon; launch from a macOS app bundle or other GUI host and verify the visible window.",
    }
}

fn needs_legacy_macos_lwjgl2_patch(version: &VersionJson, platform: Platform) -> bool {
    platform.os == Os::MacOs
        && platform.arch == Arch::Aarch64
        && version
            .libraries
            .iter()
            .any(|library| library.name.starts_with("org.lwjgl.lwjgl:lwjgl:"))
}

fn needs_macos_arm64_lwjgl3_patch(version: &VersionJson, platform: Platform) -> bool {
    platform.os == Os::MacOs
        && platform.arch == Arch::Aarch64
        && version
            .libraries
            .iter()
            .any(|library| library.name.starts_with("org.lwjgl:"))
        && !version.libraries.iter().any(|library| {
            library.name.contains("3.3.1-mmachina.1")
                || library.name.contains(":natives-macos-arm64")
                || library.name.contains(":natives-osx-arm64")
        })
}

fn apply_legacy_macos_lwjgl2_patch(version: &VersionJson) -> VersionJson {
    let mut patched = version.clone();
    patched
        .libraries
        .retain(|library| !is_legacy_lwjgl2_replaced_library(&library.name));
    patched.libraries.extend(legacy_macos_lwjgl2_libraries());
    patched
}

fn apply_macos_arm64_lwjgl3_patch(version: &VersionJson) -> VersionJson {
    let mut patched = version.clone();
    patched
        .libraries
        .retain(|library| !is_lwjgl3_replaced_library(&library.name));
    patched.libraries.extend(macos_arm64_lwjgl3_libraries());
    patched
}

fn is_legacy_lwjgl2_replaced_library(name: &str) -> bool {
    [
        "org.lwjgl.lwjgl:",
        "net.java.jinput:",
        "net.java.jutils:",
        "ca.weblite:java-objc-bridge:",
        "com.mojang:text2speech:",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

fn is_lwjgl3_replaced_library(name: &str) -> bool {
    name.starts_with("org.lwjgl:") || name.starts_with("ca.weblite:java-objc-bridge:")
}

fn legacy_macos_lwjgl2_libraries() -> Vec<Library> {
    vec![
        artifact_library(
            "com.mojang:text2speech:1.11.3",
            "https://libraries.minecraft.net/com/mojang/text2speech/1.11.3/text2speech-1.11.3.jar",
            "f378f889797edd7df8d32272c06ca80a1b6b0f58",
            13164,
            None,
        ),
        artifact_library(
            "ca.weblite:java-objc-bridge:1.1.0-mmachina.1",
            "https://github.com/MinecraftMachina/Java-Objective-C-Bridge/releases/download/1.1.0-mmachina.1/java-objc-bridge-1.1.jar",
            "369a83621e3c65496348491e533cb97fe5f2f37d",
            91947,
            None,
        ),
        native_library(
            "net.java.jinput:jinput-platform:2.0.5",
            "net/java/jinput/jinput-platform/2.0.5/jinput-platform-2.0.5-natives-osx.jar",
            "https://libraries.minecraft.net/net/java/jinput/jinput-platform/2.0.5/jinput-platform-2.0.5-natives-osx.jar",
            "53f9c919f34d2ca9de8c51fc4e1e8282029a9232",
            12186,
        ),
        artifact_library(
            "net.java.jinput:jinput:2.0.5",
            "https://libraries.minecraft.net/net/java/jinput/jinput/2.0.5/jinput-2.0.5.jar",
            "39c7796b469a600f72380316f6b1f11db6c2c7c4",
            208338,
            None,
        ),
        artifact_library(
            "net.java.jutils:jutils:1.0.0",
            "https://libraries.minecraft.net/net/java/jutils/jutils/1.0.0/jutils-1.0.0.jar",
            "e12fe1fda814bd348c1579329c86943d2cd3c6a6",
            7508,
            None,
        ),
        lwjgl_platform_library(),
        artifact_library(
            "org.lwjgl.lwjgl:lwjgl:2.9.4-nightly-20150209",
            "https://libraries.minecraft.net/org/lwjgl/lwjgl/lwjgl/2.9.4-nightly-20150209/lwjgl-2.9.4-nightly-20150209.jar",
            "697517568c68e78ae0b4544145af031c81082dfe",
            1047168,
            None,
        ),
        artifact_library(
            "org.lwjgl.lwjgl:lwjgl_util:2.9.4-nightly-20150209",
            "https://libraries.minecraft.net/org/lwjgl/lwjgl/lwjgl_util/2.9.4-nightly-20150209/lwjgl_util-2.9.4-nightly-20150209.jar",
            "d51a7c040a721d13efdfbd34f8b257b2df882ad0",
            173887,
            None,
        ),
    ]
}

fn macos_arm64_lwjgl3_libraries() -> Vec<Library> {
    vec![
        artifact_library(
            "ca.weblite:java-objc-bridge:1.1.0-mmachina.1",
            "https://github.com/MinecraftMachina/Java-Objective-C-Bridge/releases/download/1.1.0-mmachina.1/java-objc-bridge-1.1.jar",
            "369a83621e3c65496348491e533cb97fe5f2f37d",
            91947,
            None,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl-glfw",
            "e9a101bca4fa30d26b21b526ff28e7c2d8927f1b",
            130128,
            "71d793d0a5a42e3dfe78eb882abc2523a2c6b496",
            129076,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl-jemalloc",
            "4fb94224378d3588d52d2beb172f2eeafea2d546",
            36976,
            "b0be721188d2e7195798780b1c5fe7eafe8091c1",
            103478,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl-openal",
            "d48e753d85916fc8a200ccddc709b36e3865cc4e",
            88880,
            "6b80fc0b982a0723b141e88859c42d6f71bd723f",
            346131,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl-opengl",
            "962c2a8d2a8cdd3b89de3d78d766ab5e2133c2f4",
            929233,
            "bb575058e0372f515587b5d2d04ff7db185f3ffe",
            41667,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl-stb",
            "703e4b533e2542560e9f94d6d8bd148be1c1d572",
            113273,
            "98f0ad956c754723ef354d50057cc30417ef376a",
            178409,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl-tinyfd",
            "1203660b3131cbb8681b17ce6437412545be95e0",
            6802,
            "015b931a2daba8f0c317d84c9d14e8e98ae56e0c",
            41384,
        ),
        lwjgl3_macos_arm64_library(
            "lwjgl",
            "8e664dd69ad7bbcf2053da23efc7848e39e498db",
            719038,
            "984df31fadaab86838877b112e5b4e4f68a00ccf",
            42693,
        ),
    ]
}

fn artifact_library(
    name: &str,
    url: &str,
    sha1: &str,
    size: i64,
    path_override: Option<&str>,
) -> Library {
    let path = path_override.map(ToOwned::to_owned).unwrap_or_else(|| {
        MavenCoordinate::parse(name)
            .expect("static coordinate")
            .artifact_path()
            .to_string_lossy()
            .to_string()
    });
    Library {
        name: name.to_string(),
        url: None,
        rules: Vec::new(),
        downloads: Some(LibraryDownloads {
            artifact: Some(LibraryArtifact {
                path,
                url: url.to_string(),
                sha1: sha1.to_string(),
                size,
            }),
            classifiers: HashMap::new(),
        }),
        natives: None,
        extract: None,
    }
}

fn lwjgl3_macos_arm64_library(
    artifact: &str,
    artifact_sha1: &str,
    artifact_size: i64,
    native_sha1: &str,
    native_size: i64,
) -> Library {
    let name = format!("org.lwjgl:{artifact}:3.3.1-mmachina.1");
    let artifact_path = MavenCoordinate::parse(&name)
        .expect("static coordinate")
        .artifact_path()
        .to_string_lossy()
        .to_string();
    let native_path = format!(
        "org/lwjgl/{artifact}/3.3.1-mmachina.1/{artifact}-3.3.1-mmachina.1-natives-macos.jar"
    );
    let release_base =
        "https://github.com/MinecraftMachina/lwjgl3/releases/download/3.3.1-mmachina.1";

    let mut classifiers = HashMap::new();
    classifiers.insert(
        "natives-macos".to_string(),
        LibraryArtifact {
            path: native_path,
            url: format!("{release_base}/{artifact}-natives-macos-arm64.jar"),
            sha1: native_sha1.to_string(),
            size: native_size,
        },
    );
    let mut natives = HashMap::new();
    natives.insert("osx".to_string(), "natives-macos".to_string());
    let mut extract = HashMap::new();
    extract.insert("exclude".to_string(), vec!["META-INF/".to_string()]);

    Library {
        name,
        url: None,
        rules: Vec::new(),
        downloads: Some(LibraryDownloads {
            artifact: Some(LibraryArtifact {
                path: artifact_path,
                url: format!("{release_base}/{artifact}.jar"),
                sha1: artifact_sha1.to_string(),
                size: artifact_size,
            }),
            classifiers,
        }),
        natives: Some(natives),
        extract: Some(extract),
    }
}

fn native_library(name: &str, path: &str, url: &str, sha1: &str, size: i64) -> Library {
    let mut classifiers = HashMap::new();
    classifiers.insert(
        "natives-osx".to_string(),
        LibraryArtifact {
            path: path.to_string(),
            url: url.to_string(),
            sha1: sha1.to_string(),
            size,
        },
    );
    let mut natives = HashMap::new();
    natives.insert("osx".to_string(), "natives-osx".to_string());
    let mut extract = HashMap::new();
    extract.insert("exclude".to_string(), vec!["META-INF/".to_string()]);

    Library {
        name: name.to_string(),
        url: None,
        rules: Vec::new(),
        downloads: Some(LibraryDownloads {
            artifact: None,
            classifiers,
        }),
        natives: Some(natives),
        extract: Some(extract),
    }
}

fn lwjgl_platform_library() -> Library {
    let mut library = native_library(
        "org.lwjgl.lwjgl:lwjgl-platform:2.9.4-nightly-20150209-mmachina.2",
        "org/lwjgl/lwjgl/lwjgl-platform/2.9.4-nightly-20150209/lwjgl-platform-2.9.4-nightly-20150209-natives-osx.jar",
        "https://github.com/MinecraftMachina/lwjgl/releases/download/2.9.4-20150209-mmachina.2/lwjgl-platform-2.9.4-nightly-20150209-natives-osx.jar",
        "eff546c0b319d6ffc7a835652124c18089c67f36",
        488316,
    );
    if let Some(downloads) = &mut library.downloads {
        downloads.artifact = Some(LibraryArtifact {
            path: "org/lwjgl/lwjgl/lwjgl-platform/2.9.4-nightly-20150209/lwjgl-platform-2.9.4-nightly-20150209.jar".to_string(),
            url: "https://libraries.minecraft.net/org/lwjgl/lwjgl/lwjgl-platform/2.9.4-nightly-20150209/lwjgl-platform-2.9.4-nightly-20150209.jar".to_string(),
            sha1: "b04f3ee8f5e43fa3b162981b50bb72fe1acabb33".to_string(),
            size: 22,
        });
    }
    library
}
