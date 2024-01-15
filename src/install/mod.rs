use std::{collections::HashMap, fs, path::Path};

use crate::{
    runtime::install_jvm_runtime,
    types::{
        install_types::AssetsJson,
        shared_types::{ClientJson, ClientJsonLibrary, VersionListManifestJson},
        CallbackDict, MinecraftOptions,
    },
    utils::{
        helper::{
            check_path_inside_minecraft_directory, download_file, get_requests_response_cache,
            inherit_json, parse_rule_list,
        },
        natives::{extract_natives_file, get_natives},
    },
};

fn install_libraries(
    id: &str,
    libraries: &Vec<ClientJsonLibrary>,
    path: impl AsRef<Path>,
    callback: &CallbackDict,
) {
    let session = reqwest::blocking::Client::new();
    if let Some(set_states) = callback.set_status {
        set_states("Download Libraries".to_string());
    }

    if let Some(set_max) = callback.set_max {
        set_max(libraries.len() as i32 - 1);
    }

    for (count, i) in libraries.iter().enumerate() {
        if let Some(rules) = &i.rules {
            if !parse_rule_list(rules, &MinecraftOptions::default()) {
                continue;
            }
        }

        let mut current_path = path.as_ref().join("libraries");
        let mut download_url: String = if let Some(url) = &i.url {
            if url.ends_with('/') {
                url[..url.len() - 1].to_string()
            } else {
                url.to_string()
            }
        } else {
            "https://libraries.minecraft.net".to_string()
        };
        let (lib_path, name, mut version) = match &i.name {
            Some(s) => match s.split(':').collect::<Vec<_>>().as_slice() {
                [lib_path, name, version] => {
                    (lib_path.to_string(), name.to_string(), version.to_string())
                }
                _ => continue,
            },
            None => continue,
        };

        for lib_part in lib_path.split('.') {
            current_path = current_path.join(lib_part);
            download_url = format!("{}/{}", download_url, lib_part);
        }

        let mut fileend = "jar".to_string();
        let (ve, fi) = match version.split('@').collect::<Vec<_>>().as_slice() {
            [v, fe] => (v.to_string(), fe.to_string()),
            _ => ("".to_string(), "".to_string()),
        };
        if !ve.is_empty() {
            version = ve;
        }
        if !fi.is_empty() {
            fileend = fi;
        }

        download_url = format!("{}/{}/{}", download_url, name, version);
        current_path = current_path.join(&name).join(&version);
        let native = get_natives(i);

        let mut jar_filename_native = String::new();
        if !native.is_empty() {
            jar_filename_native = format!("{}-{}-{}.jar", name, version, native);
        }
        let jar_filename = format!("{}-{}.{}", name, version, fileend);
        download_url = format!("{}/{}", download_url, jar_filename);

        let _ = download_file(
            &download_url,
            &current_path.join(&jar_filename),
            None,
            false,
            Some(&path),
            Some(&session),
            callback,
        );

        if i.downloads.is_none() {
            if let Some(extract) = &i.extract {
                let _ = extract_natives_file(
                    &current_path.join(&jar_filename_native),
                    path.as_ref().join("versions").join(id).join("natives"),
                    extract,
                );
            }
            continue;
        }

        if let Some(artifact) = &i.downloads.as_ref().unwrap().artifact {
            if !artifact.url.is_empty() && !artifact.path.is_empty() {
                let _ = download_file(
                    &artifact.url,
                    path.as_ref().join("libraries").join(&artifact.path),
                    Some(&artifact.sha1),
                    false,
                    Some(&path),
                    Some(&session),
                    callback,
                );
            }
        }

        if !native.is_empty() {
            if let Some(classifiers) = &i.downloads.as_ref().unwrap().classifiers {
                if let Some(native_download) = classifiers.get(&native) {
                    let _ = download_file(
                        &native_download.url,
                        &current_path.join(&jar_filename_native),
                        Some(&native_download.sha1),
                        false,
                        Some(&path),
                        Some(&session),
                        callback,
                    );
                }
                let _ = extract_natives_file(
                    &current_path.join(&jar_filename_native),
                    path.as_ref().join("versions").join(id).join("natives"),
                    i.extract
                        .as_ref()
                        .unwrap_or(&HashMap::from([("exclude".to_string(), Vec::new())])),
                );
            }
        }
        if let Some(set_progress) = callback.set_progress {
            set_progress(count as i32);
        }
    }
}

fn install_assets(
    data: &ClientJson,
    path: impl AsRef<Path>,
    callback: &CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    if data.asset_index.is_none() {
        return Ok(());
    }
    if let Some(set_states) = callback.set_status {
        set_states("Download Assets.".to_string());
    }
    let session = reqwest::blocking::Client::new();
    let local_path = path
        .as_ref()
        .join("assets")
        .join("indexes")
        .join(data.assets.clone().unwrap() + ".json");
    let _ = download_file(
        &data.asset_index.clone().unwrap().url,
        &local_path,
        Some(&data.asset_index.clone().unwrap().sha1),
        false,
        None::<&Path>,
        Some(&session),
        callback,
    );

    let file = fs::File::open(&local_path)?;
    let assets_data: AssetsJson = serde_json::from_reader(file)?;
    if let Some(set_max) = callback.set_max {
        set_max(assets_data.objects.len() as i32 - 1);
    }
    let mut count = 0;
    for value in assets_data.objects.values() {
        let url = "https://resources.download.minecraft.net/".to_owned()
            + value.hash.get(..2).unwrap()
            + "/"
            + &value.hash;
        let _ = download_file(
            &url,
            &path
                .as_ref()
                .join("assets")
                .join("objects")
                .join(value.hash.get(..2).unwrap())
                .join(&value.hash),
            Some(&value.hash),
            false,
            Some(&path),
            Some(&session),
            callback,
        );
        count += 1;
        if let Some(set_progress) = callback.set_progress {
            set_progress(count);
        }
    }
    Ok(())
}

fn do_version_install(
    version_id: &str,
    path: impl AsRef<Path>,
    url: Option<&str>,
    sha1: Option<&str>,
    depth: i32,
    callback: &CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    if depth > 8 {
        return Err("so much inherit".into());
    }
    let version_path = path
        .as_ref()
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id));
    if let Some(url) = url {
        let _ = download_file(url, &version_path, sha1, false, Some(&path), None, callback);
    }
    let file = fs::File::open(&version_path)?;
    let mut version_data: ClientJson = serde_json::from_reader(file)?;
    // for forge
    if let Some(inherits_from) = &version_data.inherits_from {
        let _ = install_minecraft_version_dfs(&inherits_from, path.as_ref(), depth, callback);
        version_data = inherit_json(&version_data, &path)?;
    }

    install_libraries(
        &version_data.id.clone().unwrap_or("".to_string()),
        &version_data.libraries.clone().unwrap_or(Vec::new()),
        &path,
        callback,
    );
    let _ = install_assets(&version_data, &path, callback);
    // download minecraft.jar
    let mcjar_path = path
        .as_ref()
        .join("versions")
        .join(version_data.id.clone().unwrap())
        .join(format!("{}.jar", version_data.id.clone().unwrap()));
    if let Some(downloads) = &version_data.downloads {
        let _ = download_file(
            &downloads.get("client").unwrap().url,
            &mcjar_path,
            Some(&downloads.get("client").unwrap().sha1),
            false,
            Some(&path),
            None,
            callback,
        );
    }
    // need to copy jar for old forge versions
    if let Some(inherits_from) = &version_data.inherits_from {
        if !mcjar_path.is_file() {
            let inherits_path = path
                .as_ref()
                .join("versions")
                .join(inherits_from)
                .join(format!("{}.jar", inherits_from));
            check_path_inside_minecraft_directory(&path, &inherits_path)?;
            let _ = fs::copy(mcjar_path, inherits_path);
        }
    }
    // install java runtime if needed
    if let Some(java_version) = &version_data.java_version {
        if let Some(set_states) = callback.set_status {
            set_states("install java runtime.".to_string());
        }
        let _ = install_jvm_runtime(&java_version.component, &path, callback);
    }
    Ok(())
}

fn install_minecraft_version_dfs(
    version_id: &str,
    minecraft_directory: impl AsRef<Path>,
    depth: i32,
    callback: &CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    if minecraft_directory
        .as_ref()
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id))
        .is_file()
    {
        let _ = do_version_install(
            version_id,
            &minecraft_directory,
            None,
            None,
            depth + 1,
            callback,
        );
        return Ok(());
    }
    let response = get_requests_response_cache(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
    )?;
    let version_list: VersionListManifestJson = serde_json::from_str(&response)?;
    for i in version_list.versions {
        if i.id == version_id {
            let _ = do_version_install(
                version_id,
                &minecraft_directory,
                Some(&i.url),
                Some(&i.sha1),
                depth + 1,
                callback,
            );
            return Ok(());
        }
    }
    Err(format!("version not found: {}", version_id).into())
}

pub fn install_minecraft_version(
    version_id: &str,
    minecraft_directory: impl AsRef<Path>,
    callback: &CallbackDict,
) -> Result<(), Box<dyn std::error::Error>> {
    install_minecraft_version_dfs(version_id, minecraft_directory, 0, callback)
}
