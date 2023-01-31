use super::installer::Installer;
use crate::installers::conda::conda_install;

use anyhow::{anyhow, Error, Result};

pub struct Pip {
    package: String,
    version: String,
    channels: Vec<String>,
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

        Ok(Pip {
            package,
            version: version.to_string(),
            channels,
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
        )?;
        let pip_path = to_dir.join("bin").join("pip");
        let mut command = std::process::Command::new(pip_path);
        command.stdout(Into::<std::process::Stdio>::into(os_pipe::dup_stderr()?));

        command.arg("install");
        command.arg(format!("{}=={}", self.package, self.version));

        let mut output = command.spawn().unwrap();
        if !output.wait()?.success() {
            return Err(anyhow!("Pip installation exited with error"));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("pip installer for {}={}", self.package, self.version)
    }
}
