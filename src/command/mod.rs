use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    types::{shared_types::ClientJson, MinecraftOptions},
    utils::{
        helper::{get_classpath_separator, get_library_path, parse_rule_list},
        natives::get_natives,
    },
};

fn get_libraries(data: &ClientJson, path: impl AsRef<Path>) -> String {
    let classpath_separator = get_classpath_separator();
    let mut libstr = String::new();
    if let Some(libraries) = &data.libraries {
        for i in libraries {
            if i.rules.is_some()
                && !parse_rule_list(&i.rules.clone().unwrap(), &MinecraftOptions::default())
            {
                continue;
            }

            libstr.push_str(&i.name.clone().unwrap_or("".to_string()));
            libstr.push_str(&classpath_separator);
            let native = get_natives(i);
            if !native.is_empty() {
                if i.downloads.is_some()
                    && i.downloads
                        .clone()
                        .unwrap()
                        .classifiers
                        .unwrap_or(HashMap::new())
                        .contains_key(&native)
                {
                    libstr.push_str(
                        path.as_ref()
                            .join("libraries")
                            .join(
                                &i.downloads
                                    .clone()
                                    .unwrap()
                                    .classifiers
                                    .unwrap()
                                    .get(&native)
                                    .unwrap()
                                    .path,
                            )
                            .to_str()
                            .unwrap(),
                    );
                    libstr.push_str(&classpath_separator);
                } else {
                    let arg_name = i.name.clone().unwrap_or("".to_string()) + "-" + &native;
                    libstr.push_str(get_library_path(&arg_name, &path).to_str().unwrap());
                    libstr.push_str(&classpath_separator);
                }
            }
        }
    }
    if let Some(jar) = &data.jar {
        let tmp_name = path
            .as_ref()
            .join("versions")
            .join(jar)
            .join(jar.to_owned() + ".jar");
        libstr.push_str(tmp_name.to_str().unwrap());
    } else {
        if let Some(id) = &data.id {
            let tmp_name = path
                .as_ref()
                .join("versions")
                .join(id)
                .join(id.to_owned() + ".jar");
            libstr.push_str(tmp_name.to_str().unwrap());
        }
    }

    libstr
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_minecraft_option_default() {
        assert_eq!(
            MinecraftOptions::default(),
            MinecraftOptions {
                username: None,
                uuid: None,
                token: None,
                executable_path: None,
                default_executable_path: None,
                jvm_arguments: None,
                launcher_name: None,
                launcher_version: None,
                game_directory: None,
                demo: None,
                custom_resolution: None,
                resolution_width: None,
                resolution_height: None,
                server: None,
                port: None,
                natives_directory: None,
                enable_logging_config: None,
                disable_multiplayer: None,
                disable_chat: None,
                quick_play_path: None,
                quick_play_singleplayer: None,
                quick_play_multiplayer: None,
                quick_play_realms: None
            }
        )
    }
}
