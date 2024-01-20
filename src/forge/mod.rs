use std::path::Path;

fn get_data_library_path(libname: &str, path: impl AsRef<Path>) -> String {
    let libname = &libname[1..libname.len() - 1];
    let mut libpath = path.as_ref().join("libraries");
    let mut parts = libname.split(":");
    let base_path = parts.next().unwrap_or_default();
    let libname = parts.next().unwrap_or_default();
    let version = parts.next().unwrap_or_default();
    let mut extra = parts.next().unwrap_or_default();
    for i in base_path.split("."){
        libpath.push(i);
    }
    let mut fileend = "jar";
    if let Some(at_index) = extra.find('@') {
        fileend = &extra[at_index + 1..];
        extra = &extra[..at_index];
    }
    libpath.push(libname);
    libpath.push(version);
    libpath.push(format!("{}-{}-{}.{}", libname, version, extra, fileend));
    libpath.to_str().unwrap_or_default().to_owned()
}

pub fn forge_to_installed_version(forge_version: &str) -> Result<String, String> {
    match forge_version.split_once("-") {
        Some((vanilla_part, forge_part)) => Ok(format!("{}-forge-{}", vanilla_part, forge_part)),
        None => Err(format!("{} is not a valid forge version", forge_version)),
    }
}
