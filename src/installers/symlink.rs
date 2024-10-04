use super::installer::Installer;

use anyhow::{anyhow, Context, Error, Result};

pub struct Symlink {
    name: String,
    version: String,
    path: std::path::PathBuf,
}

impl Symlink {
    pub fn new(
        name: &String,
        version: &str,
        app_config: &serde_yaml::Mapping,
    ) -> Result<Self, Error> {
        let path = match app_config.get("path") {
            Some(serde_yaml::Value::String(path)) => std::path::Path::new(path),
            _ => {
                return Err(anyhow!("Expected a string path in config for {}", name));
            }
        };

        Ok(Symlink {
            name: name.clone(),
            version: version.to_string(),
            path: path.to_owned(),
        })
    }
}

impl Installer for Symlink {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        std::fs::create_dir_all(to_dir)?;

        if !self.path.exists() {
            return Err(anyhow!(
                "Destination symlink {} doesn't exist",
                self.path.display()
            ));
        }

        let file = to_dir.join(&self.name);
        std::os::unix::fs::symlink(&self.path, &file).context(anyhow!(
            "Unable to symlink {} to {}",
            self.path.display(),
            file.display()
        ))
    }

    fn describe(&self) -> String {
        format!("symlink installer for {} v{}", self.name, self.version)
    }
}
