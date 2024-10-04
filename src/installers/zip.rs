use super::installer::Installer;

use anyhow::{anyhow, Context, Error, Result};

pub struct Zip {
    name: String,
    version: String,
    url: String,
}

impl Zip {
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

        Ok(Zip {
            name: name.clone(),
            version: version.to_string(),
            url,
        })
    }
}

impl Installer for Zip {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        std::fs::create_dir_all(to_dir)?;

        let response = reqwest::blocking::get(&self.url)?;
        let content = response.bytes()?.to_vec();
        zip_extract::extract(std::io::Cursor::new(content), to_dir, false)
            .context("While unzipping downloaded file")?;

        Ok(())
    }

    fn describe(&self) -> String {
        format!("zip installer for {} v{}", self.name, self.version)
    }
}
