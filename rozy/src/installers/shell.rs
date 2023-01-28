use super::installer::Installer;
use crate::utils::download_to;
use crate::utils::run_with_stderr_to_stdout;
use tempfile::tempdir;

use anyhow::{anyhow, Context, Error, Result};

pub struct Shell {
    name: String,
    version: String,
    url: String,
    extra_path_during_install: Option<String>,
    shell_args: Vec<String>,
}

impl Shell {
    pub fn new(
        name: &String,
        version: &str,
        app_config: &serde_yaml::Mapping,
    ) -> Result<Self, Error> {
        let url = match app_config.get("url") {
            Some(serde_yaml::Value::String(url)) => url.clone(),
            _ => {
                return Err(anyhow!("Expected a string url in config for {}", name));
            }
        };

        let extra_path_during_install = match app_config.get("extra_path_during_install") {
            Some(serde_yaml::Value::String(value)) => Some(value.clone()),
            _ => None,
        };

        let shell_args = match app_config.get("shell_args") {
            // TODO: Probably a way to do this without unwrapping
            Some(serde_yaml::Value::Sequence(seq)) => seq
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            _ => vec![],
        };

        Ok(Shell {
            name: name.clone(),
            version: version.to_string(),
            url,
            extra_path_during_install,
            shell_args,
        })
    }
}

impl Installer for Shell {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        std::fs::create_dir_all(to_dir)?;

        let dir = tempdir()?;
        let file = dir.path().join("installer.sh");
        download_to(&file, &self.url)?;

        let mut command = std::process::Command::new("/bin/bash");

        if let Some(extra_path) = &self.extra_path_during_install {
            let path = format!(
                "{}:{}",
                extra_path,
                std::env::var("PATH").context("While checking $PATH")?
            );
            command.env("PATH", path);
        }
        command.env("INSTALL_DIR", to_dir.as_os_str().to_str().unwrap());

        command.arg(file.as_os_str().to_str().unwrap());
        for arg in &self.shell_args {
            command.arg(arg);
        }

        let output = run_with_stderr_to_stdout(command)?;
        if !output.success() {
            return Err(anyhow!("Shell installer exited with {:?}", output));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("shell installer for {} v.{}", self.name, self.version)
    }
}
