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
