use std::collections::HashMap;

use crate::{
    core::{
        maven::MavenCoordinate,
        version::{Library, LibraryArtifact, LibraryDownloads, VersionJson},
    },
    platform::{Arch, Os, Platform},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityPolicy {
    Auto,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityPatch {
    LegacyMacArm64Lwjgl2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaRuntimeHint {
    pub major_version: i32,
    pub arch: Arch,
    pub distribution_hint: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompatibilityResult {
    pub version: VersionJson,
    pub applied_patches: Vec<CompatibilityPatch>,
    pub java_runtime: Option<JavaRuntimeHint>,
}

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
        };
    }

    CompatibilityResult {
        version: version.clone(),
        applied_patches: Vec::new(),
        java_runtime: None,
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

fn apply_legacy_macos_lwjgl2_patch(version: &VersionJson) -> VersionJson {
    let mut patched = version.clone();
    patched
        .libraries
        .retain(|library| !is_legacy_lwjgl2_replaced_library(&library.name));
    patched.libraries.extend(legacy_macos_lwjgl2_libraries());
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
