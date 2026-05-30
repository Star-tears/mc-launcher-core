//! Platform detection and Minecraft metadata naming helpers.

/// Operating system family used by rule and native-library selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// Microsoft Windows.
    Windows,
    /// macOS, named `osx` in Minecraft metadata.
    MacOs,
    /// Linux.
    Linux,
    /// Any unsupported or unknown operating system.
    Other,
}

/// CPU architecture family used by rule and native-library selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// 32-bit x86.
    X86,
    /// 64-bit x86.
    X86_64,
    /// 64-bit ARM.
    Aarch64,
    /// Any unsupported or unknown architecture.
    Other,
}

/// Operating-system and architecture pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    /// Operating system family.
    pub os: Os,
    /// CPU architecture family.
    pub arch: Arch,
}

impl Platform {
    /// Detects the platform of the current process.
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

    /// Returns the operating-system name used in Minecraft metadata rules.
    pub fn minecraft_os_name(self) -> &'static str {
        match self.os {
            Os::Windows => "windows",
            Os::MacOs => "osx",
            Os::Linux => "linux",
            Os::Other => "unknown",
        }
    }

    /// Returns true when the platform is 32-bit x86.
    pub fn is_32_bit(self) -> bool {
        self.arch == Arch::X86
    }
}
