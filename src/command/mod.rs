use std::{collections::HashMap, fs, path::Path};

use crate::{
    runtime::get_executable_path,
    types::{
        shared_types::{ClientJson, StringAndClientJsonArgumentRuleValue, StringAndVecStringValue},
        MinecraftOptions,
    },
    utils::{
        get_core_version,
        helper::{get_classpath_separator, get_library_path, inherit_json, parse_rule_list},
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
            let arg_name = i.name.clone().unwrap_or("".to_string());
            libstr.push_str(get_library_path(&arg_name, &path).to_str().unwrap());
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

fn get_arguments_string(
    version_data: &ClientJson,
    path: impl AsRef<Path>,
    options: &MinecraftOptions,
    classpath: impl AsRef<Path>,
) -> Vec<String> {
    let mut arglist: Vec<String> = Vec::new();

    for v in version_data
        .minecraft_arguments
        .as_ref()
        .unwrap_or(&"".to_string())
        .split(' ')
    {
        let v = replace_arguments(
            v.to_string(),
            version_data,
            path.as_ref(),
            options,
            classpath.as_ref(),
        );
        arglist.push(v);
    }

    // Custom resolution is not in the list
    if options.custom_resolution.unwrap_or(false) {
        arglist.push("--width".to_string());
        arglist.push(
            options
                .resolution_width
                .as_deref()
                .unwrap_or("854")
                .to_string(),
        );
        arglist.push("--height".to_string());
        arglist.push(
            options
                .resolution_height
                .as_deref()
                .unwrap_or("480")
                .to_string(),
        );
    }

    if options.demo.unwrap_or(false) {
        arglist.push("--demo".to_string());
    }

    arglist
}

fn get_arguments(
    data: Vec<StringAndClientJsonArgumentRuleValue>,
    version_data: &ClientJson,
    path: impl AsRef<Path>,
    options: &MinecraftOptions,
    classpath: impl AsRef<Path>,
) -> Vec<String> {
    let mut arglist: Vec<String> = Vec::new();

    for i in data {
        match i {
            StringAndClientJsonArgumentRuleValue::StringValue(s) => {
                arglist.push(replace_arguments(
                    s,
                    version_data,
                    &path,
                    options,
                    &classpath,
                ));
            }
            StringAndClientJsonArgumentRuleValue::ClientJsonArgumentRuleValue(rule) => {
                if let Some(compatibility_rules) = rule.compatibility_rules.as_ref() {
                    if !parse_rule_list(compatibility_rules, options) {
                        continue;
                    }
                }

                if let Some(rules) = rule.rules.as_ref() {
                    if !parse_rule_list(rules, options) {
                        continue;
                    }
                }

                if let Some(value) = rule.value.as_ref() {
                    match value {
                        StringAndVecStringValue::StringValue(s) => {
                            arglist.push(replace_arguments(
                                s.to_string(),
                                version_data,
                                &path,
                                options,
                                &classpath,
                            ));
                        }
                        StringAndVecStringValue::VecStringValue(vec_s) => {
                            for s in vec_s {
                                arglist.push(replace_arguments(
                                    s.to_string(),
                                    version_data,
                                    &path,
                                    options,
                                    &classpath,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    arglist
}

pub fn get_minecraft_command(
    version: &str,
    minecraft_directory: impl AsRef<Path>,
    options_arg: &MinecraftOptions,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if !minecraft_directory
        .as_ref()
        .join("versions")
        .join(version)
        .is_dir()
    {
        return Err(format!("version not found: {}", version).into());
    }
    let mut options = options_arg.clone();

    let file_path = minecraft_directory
        .as_ref()
        .join("versions")
        .join(version)
        .join(version.to_owned() + ".json");
    let json_content = fs::read_to_string(&file_path)?;
    let mut data: ClientJson = serde_json::from_str(&json_content)?;

    if data.inherits_from.is_some() {
        data = inherit_json(&data, &minecraft_directory)?;
    }

    if options.natives_directory.is_none() {
        let version_id = data.id.as_ref().unwrap();
        let default_natives_directory = minecraft_directory
            .as_ref()
            .join("versions")
            .join(version_id)
            .join("natives");
        options.natives_directory = Some(default_natives_directory.to_str().unwrap().to_string());
    }
    let classpath = get_libraries(&data, &minecraft_directory);

    let mut command: Vec<String> = Vec::new();

    if let Some(executable_path) = options.executable_path.as_ref() {
        command.push(executable_path.to_string());
    } else if let Some(java_version) = data.java_version.as_ref() {
        match get_executable_path(&java_version.component, &minecraft_directory) {
            Some(java_path) => command.push(java_path.to_str().unwrap().to_string()),
            None => command.push("java".to_owned()),
        }
    } else {
        command.push(
            options
                .default_executable_path
                .clone()
                .unwrap_or("java".to_owned()),
        );
    }

    if let Some(jvm_arguments) = options.jvm_arguments.as_ref() {
        command.extend(jvm_arguments.clone());
    }

    match data.arguments.as_ref() {
        Some(arguments) => match arguments.get("jvm") {
            Some(v) => {
                command.extend(get_arguments(
                    v.to_vec(),
                    &data,
                    &minecraft_directory,
                    &options,
                    &classpath,
                ));
            }
            None => {
                command.push(format!(
                    "-Djava.library.path={}",
                    options.natives_directory.clone().unwrap_or("".to_string())
                ));
                command.push("-cp".to_string());
                command.push(classpath.clone());
            }
        },
        None => {
            command.push(format!(
                "-Djava.library.path={}",
                options.natives_directory.clone().unwrap_or("".to_string())
            ));
            command.push("-cp".to_string());
            command.push(classpath.clone());
        }
    }
    if options.enable_logging_config.unwrap_or(false) {
        if let Some(logging) = data.logging.as_ref() {
            if !logging.is_empty() {
                let logger_file = minecraft_directory
                    .as_ref()
                    .join("assets")
                    .join("log_configs")
                    .join(logging.get("client").unwrap().file.id.clone());
                command.push(
                    logging
                        .get("client")
                        .unwrap()
                        .argument
                        .replace("${path}", logger_file.to_str().unwrap()),
                );
            }
        }
    }

    if let Some(main_class) = data.main_class.as_ref() {
        command.push(main_class.clone());
    }

    if data.minecraft_arguments.is_some() {
        command.extend(get_arguments_string(
            &data,
            minecraft_directory,
            &options,
            &classpath,
        ));
    } else if let Some(arguments) = data.arguments.as_ref() {
        if let Some(game) = arguments.get("game") {
            command.extend(get_arguments(
                game.to_vec(),
                &data,
                minecraft_directory,
                &options,
                &classpath,
            ));
        }
    }

    if let Some(server) = options.server.as_ref() {
        command.push("--server".to_string());
        command.push(server.to_string());
        if let Some(port) = options.port.as_ref() {
            command.push("--port".to_string());
            command.push(port.to_string());
        }
    }

    if options.disable_multiplayer.unwrap_or(false) {
        command.push("--disableMultiplayer".to_string());
    }

    if options.disable_chat.unwrap_or(false) {
        command.push("--disableChat".to_string());
    }

    Ok(command)
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
