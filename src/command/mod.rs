use std::{collections::HashMap, path::Path};

use crate::{
    types::{shared_types::ClientJson, MinecraftOptions},
    utils::{
        get_core_version,
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

fn replace_arguments(
    mut argstr: String,
    version_data: &ClientJson,
    path: impl AsRef<Path>,
    options: &MinecraftOptions,
    classpath: impl AsRef<Path>,
) -> String {
    argstr = argstr.replace(
        "${natives_directory}",
        options.natives_directory.as_deref().unwrap_or(""),
    );
    argstr = argstr.replace(
        "${launcher_name}",
        options
            .launcher_name
            .as_deref()
            .unwrap_or("mc-launcher-core"),
    );
    argstr = argstr.replace(
        "${launcher_version}",
        options
            .launcher_version
            .as_deref()
            .unwrap_or(&get_core_version()),
    );
    argstr = argstr.replace(
        "${classpath}",
        classpath.as_ref().to_str().unwrap_or_default(),
    );
    argstr = argstr.replace(
        "${auth_player_name}",
        options.username.as_deref().unwrap_or("{username}"),
    );
    argstr = argstr.replace(
        "${version_name}",
        version_data.id.as_deref().unwrap_or_default(),
    );
    argstr = argstr.replace(
        "${game_directory}",
        options
            .game_directory
            .as_deref()
            .unwrap_or(path.as_ref().to_str().unwrap_or_default()),
    );
    argstr = argstr.replace(
        "${assets_root}",
        path.as_ref().join("assets").to_str().unwrap_or_default(),
    );
    argstr = argstr.replace(
        "${assets_index_name}",
        version_data
            .assets
            .as_deref()
            .unwrap_or(&version_data.id.as_ref().unwrap_or(&"".to_string())),
    );
    argstr = argstr.replace("${auth_uuid}", options.uuid.as_deref().unwrap_or("{uuid}"));
    argstr = argstr.replace(
        "${auth_access_token}",
        options.token.as_deref().unwrap_or("{token}"),
    );
    argstr = argstr.replace("${user_type}", "msa");
    argstr = argstr.replace(
        "${version_type}",
        version_data.r#type.as_deref().unwrap_or_default(),
    );
    argstr = argstr.replace("${user_properties}", "{}");
    argstr = argstr.replace(
        "${resolution_width}",
        options.resolution_width.as_deref().unwrap_or("854"),
    );
    argstr = argstr.replace(
        "${resolution_height}",
        options.resolution_height.as_deref().unwrap_or("480"),
    );
    argstr = argstr.replace(
        "${game_assets}",
        path.as_ref()
            .join("assets")
            .join("virtual")
            .join("legacy")
            .to_str()
            .unwrap_or_default(),
    );
    argstr = argstr.replace(
        "${auth_session}",
        options.token.as_deref().unwrap_or("{token}"),
    );
    argstr = argstr.replace(
        "${library_directory}",
        path.as_ref().join("libraries").to_str().unwrap_or_default(),
    );
    argstr = argstr.replace("${classpath_separator}", &get_classpath_separator());
    argstr = argstr.replace(
        "${quickPlayPath}",
        options
            .quick_play_path
            .as_deref()
            .unwrap_or("{quickPlayPath}"),
    );
    argstr = argstr.replace(
        "${quickPlaySingleplayer}",
        options
            .quick_play_singleplayer
            .as_deref()
            .unwrap_or("{quickPlaySingleplayer}"),
    );
    argstr = argstr.replace(
        "${quickPlayMultiplayer}",
        options
            .quick_play_multiplayer
            .as_deref()
            .unwrap_or("{quickPlayMultiplayer}"),
    );
    argstr = argstr.replace(
        "${quickPlayRealms}",
        options
            .quick_play_realms
            .as_deref()
            .unwrap_or("{quickPlayRealms}"),
    );
    argstr
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
