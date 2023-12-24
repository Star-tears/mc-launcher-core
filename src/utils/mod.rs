use std::env;
use std::path::PathBuf;

fn get_minecraft_directory() -> PathBuf {
    let os = env::consts::OS;
    if os == "windows" {
        // Windows
        let home = env::var("APPDATA").unwrap_or_else(|_| {
            // If APPDATA is not set, use the default path
            let home = env::var("USERPROFILE").expect("USERPROFILE is not set");
            home
        });
        let appdata = format!(r"{}\.minecraft", home);
        PathBuf::from(appdata)
    } else if os == "macos" {
        // MacOS
        let home = env::var("HOME").expect("HOME is not set");
        PathBuf::from(format!("{}/Library/Application Support/minecraft", home))
    } else {
        // Other platforms (Linux and others)
        let home = env::var("HOME").expect("HOME is not set");
        PathBuf::from(format!("{}/.minecraft", home))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_minecraft_directory() {
        let minecraft_directory = get_minecraft_directory();
        println!("Minecraft directory: {:?}", minecraft_directory);
    }
}
