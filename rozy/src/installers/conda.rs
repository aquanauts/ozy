use super::installer::{run_subcommand_for_installer, Installer};
use crate::files::delete_if_exists;
use anyhow::{anyhow, Error, Result};

use tempfile::tempdir;

pub struct Conda {
    package: String,
    version: String,
    channels: Vec<String>,
    conda_bin: String,
    pyinstaller: bool,
}

pub fn conda_install(
    conda_bin: &str,
    channels: &[String],
    to_dir: &std::path::Path,
    to_install: &[String],
) -> Result<()> {
    if let Some(parent) = to_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Micromamba installer requires this path to be free
    delete_if_exists(to_dir)?;

    let conda_cache_dir = tempdir()?;
    let env = vec![("CONDA_PKGS_DIRS", conda_cache_dir.path().to_str().unwrap())];

    let mut args = vec!["create", "-y", "-q", "-p", to_dir.to_str().unwrap()];
    for arg in channels.iter() {
        args.push("-c");
        args.push(arg);
    }

    for arg in to_install.iter() {
        args.push(arg);
    }

    let subcommand_result =
        run_subcommand_for_installer(conda_bin, args.into_iter(), env.into_iter());
    if subcommand_result.is_err() {
        return Err(anyhow!("Conda installation exited with error"));
    }

    Ok(())
}

impl Conda {
    pub fn new(_: &str, version: &str, app_config: &serde_yaml::Mapping) -> Result<Self, Error> {
        let package = app_config["package"].as_str().unwrap().to_string();
        let channels = match app_config.get("channels") {
            // TODO: Probably a way to do this without unwrapping
            Some(serde_yaml::Value::Sequence(seq)) => seq
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            _ => vec![],
        };
        let conda_bin = match app_config.get("conda_bin") {
            Some(serde_yaml::Value::String(value)) => value.to_owned(),
            _ => String::from("conda"),
        };

        let pyinstaller = match app_config.get("pyinstaller") {
            Some(serde_yaml::Value::Bool(value)) => value.to_owned(),
            _ => false,
        };

        Ok(Conda {
            package,
            version: version.to_string(),
            channels,
            conda_bin,
            pyinstaller,
        })
    }
}

impl Installer for Conda {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());

        if self.pyinstaller {
            return Err(anyhow!("Pyinstaller mode not yet implemented"));
        }
        let versioned_package = format!("{}={}", self.package, self.version);
        conda_install(
            &self.conda_bin,
            &self.channels,
            to_dir,
            &[versioned_package],
        )?;
        Ok(())
    }

    fn describe(&self) -> String {
        format!("conda installer for {}={}", self.package, self.version)
    }
}
