use std::env;

fn get_jvm_platform_string() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86") => "windows-x86".to_string(),
        ("windows", _) => "windows-x64".to_string(),
        ("linux", "x86") => "linux-i386".to_string(),
        ("linux", _) => "linux".to_string(),
        ("macos", "aarch64") => "mac-os-arm64".to_string(),
        ("macos", _) => "mac-os".to_string(),
        _ => "gamecore".to_string(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn debug_get_jvm_platform_string() {
        println!("{}", get_jvm_platform_string());
    }
}
