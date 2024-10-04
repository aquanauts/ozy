use super::installer::Installer;
use crate::installers::{conda::conda_install, installer::run_subcommand_for_installer};
use anyhow::{anyhow, Error, Result};

pub struct Pip {
    package: String,
    version: String,
    channels: Vec<String>,
    env: Vec<(String, String)>,
}

impl Pip {
    pub fn new(_: &String, version: &str, app_config: &serde_yaml::Mapping) -> Result<Self, Error> {
        let package = app_config["package"].as_str().unwrap().to_string();
        let channels = match app_config.get("channels") {
            // TODO: Probably a way to do this without unwrapping
            Some(serde_yaml::Value::Sequence(seq)) => seq
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            _ => vec![],
        };

        let env = match app_config.get("env") {
            Some(serde_yaml::Value::Mapping(env_map)) => env_map
                .iter()
                .map(|(a, b)| {
                    (
                        a.as_str().unwrap().to_owned(),
                        b.as_str().unwrap().to_owned(),
                    )
                })
                .collect(),
            _ => vec![],
        };

        Ok(Pip {
            package,
            version: version.to_string(),
            channels,
            env,
        })
    }
}

impl Installer for Pip {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        std::fs::create_dir_all(to_dir)?;

        conda_install(
            &String::from("conda"),
            &self.channels,
            to_dir,
            &[String::from("pip")],
            &self.env,
        )?;

        let pip_command = to_dir.join("bin").join("pip");
        let version_arg = format!("{}=={}", self.package, self.version);
        let args = vec!["install", &version_arg];

        let subcommand_result = run_subcommand_for_installer(
            pip_command.to_str().unwrap(),
            args.into_iter(),
            std::iter::empty(),
        );
        if subcommand_result.is_err() {
            return Err(anyhow!("Pip installation exited with error"));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("pip installer for {}={}", self.package, self.version)
    }
}
