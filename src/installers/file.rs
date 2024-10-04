use crate::utils::download_to;

use super::installer::Installer;

use anyhow::{anyhow, Error, Result};
use std::os::unix::fs::PermissionsExt;

pub struct File {
    name: String,
    version: String,
    url: String,
}

impl File {
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

        Ok(File {
            name: name.clone(),
            version: version.to_string(),
            url,
        })
    }
}

impl Installer for File {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        std::fs::create_dir_all(to_dir)?;

        let file = to_dir.join(&self.name);
        download_to(&file, &self.url)?;
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o774)).unwrap();

        Ok(())
    }

    fn describe(&self) -> String {
        format!("file installer for {} v{}", self.name, self.version)
    }
}
