use crate::files::get_ozy_dir;
use anyhow::{Context, Result};
use std::{collections::HashMap, time::SystemTime};

use serde_yaml::Mapping as Config;

pub fn config_mtime() -> Result<SystemTime> {
    Ok(std::fs::metadata(&get_ozy_dir()?.join("ozy.yaml"))?
        .modified()
        .unwrap())
}

pub fn load_config(base_config_filename_override: Option<&str>) -> Result<Config> {
    let mut curr_config = parse_ozy_config(
        &get_ozy_dir()?.join(base_config_filename_override.unwrap_or("ozy.yaml")),
    )?;
    let curr_dir = std::env::current_dir()?;

    let ancestors: Vec<&std::path::Path> = curr_dir.ancestors().collect();
    for dir in ancestors.iter().rev() {
        let ozy_config_path = dir.join(".ozy.yaml");
        if !ozy_config_path.exists() {
            continue;
        }

        let new_config = parse_ozy_config(&ozy_config_path)?;
        apply_overrides(&new_config, &mut curr_config);
    }

    Ok(curr_config)
}

pub fn parse_ozy_config(path: &std::path::PathBuf) -> Result<Config> {
    let raw_config = std::fs::read_to_string(path).with_context(|| "Reading config YAML")?;
    serde_yaml::from_str(&raw_config).with_context(|| "Deserializing config YAML")
}

pub fn save_ozy_user_conf(conf: &serde_yaml::Mapping) -> Result<()> {
    let path = get_ozy_dir()?.join("ozy.user.yaml");
    let file = std::fs::File::create(path)?;
    serde_yaml::to_writer(file, conf)?;
    Ok(())
}

pub fn get_ozy_user_conf() -> Result<serde_yaml::Mapping> {
    load_config(Some("ozy.user.yaml"))
}

pub fn apply_overrides(source: &serde_yaml::Mapping, dest: &mut serde_yaml::Mapping) {
    for (key, value) in source.into_iter() {
        match (value, dest.get_mut(key)) {
            (
                serde_yaml::Value::Mapping(src_child_mapping),
                Some(serde_yaml::Value::Mapping(dst_child_mapping)),
            ) => {
                apply_overrides(src_child_mapping, dst_child_mapping);
            }
            _ => {
                dest.insert(key.clone(), value.clone());
            }
        };
    }
}

fn get_candidate_substitutions(mapping: &serde_yaml::Mapping) -> HashMap<String, String> {
    let mut variables: HashMap<String, String> = HashMap::new();

    variables.insert(
        "ozy_os".to_string(),
        match std::env::consts::OS {
            "macos" => "darwin".to_string(),
            _ => std::env::consts::OS.to_string(),
        },
    );
    variables.insert(
        "ozy_arch".to_string(),
        match std::env::consts::ARCH {
            "x86_64" => "amd64".to_string(),
            _ => std::env::consts::ARCH.to_string(),
        },
    );
    variables.insert(
        "ozy_machine".to_string(),
        std::env::consts::ARCH.to_string(),
    );

    for (key, value) in mapping.into_iter() {
        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
            variables.insert(k.to_string(), v.to_string());
        }
    }

    variables
}

pub fn resolve(mapping: &mut serde_yaml::Mapping) {
    let variables = get_candidate_substitutions(mapping);

    let re = regex::Regex::new(r"\{.*?\}").unwrap();
    for (_, value) in mapping.into_iter() {
        let maybe_s = value.as_str();
        if maybe_s.is_none() {
            continue;
        }

        let mut s = maybe_s.unwrap().to_string();

        let potential_variables: Vec<String> = re
            .find_iter(&s)
            .map(|m| s[m.start() + 1..m.end() - 1].to_string())
            .collect();
        for potential_variable in potential_variables.iter() {
            let replacement: Option<&String> = variables.get(potential_variable);
            if replacement.is_none() {
                continue;
            }

            s = s.replace(&format!("{{{}}}", potential_variable), replacement.unwrap());
        }

        *value = serde_yaml::Value::String(s);
    }
}
