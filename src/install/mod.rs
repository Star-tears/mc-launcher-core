use std::{collections::HashMap, path::Path};

use crate::{
    types::{shared_types::ClientJsonLibrary, CallbackDict, MinecraftOptions},
    utils::{
        helper::{download_file, parse_rule_list},
        natives::{extract_natives_file, get_natives},
    },
};

fn install_libraries(
    id: &str,
    libraries: Vec<ClientJsonLibrary>,
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
            _ => ("".to_string(), "to".to_string()),
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
