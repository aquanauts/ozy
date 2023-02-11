use std::os::unix::fs::PermissionsExt;
use std::{fs, io};

use anyhow::{anyhow, Context, Error, Result};

use zip;

use super::installer::Installer;

pub struct SingleBinaryZip {
    name: String,
    version: String,
    url: String,
}

impl SingleBinaryZip {
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

        Ok(SingleBinaryZip {
            name: name.clone(),
            version: version.to_string(),
            url,
        })
    }
}

impl Installer for SingleBinaryZip {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        fs::create_dir_all(to_dir)?;

        let app_path = to_dir.join(&self.name);
        let response = reqwest::blocking::get(&self.url)?;
        let content = response.bytes()?.to_vec();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(content))
            .context("While unzipping downloaded file")?;
        if archive.len() != 1 {
            return Err(anyhow!("More than one file in the zipfile at {}", self.url));
        }

        let mut file = archive.by_index(0)?;
        let mut outfile = fs::File::create(&app_path)?;
        io::copy(&mut file, &mut outfile).context("While unzipping downloaded file")?;

        let mut perms = fs::metadata(&app_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&app_path, perms)?;

        Ok(())
    }

    fn describe(&self) -> String {
        format!(
            "single binary zip installer for {} v.{}",
            self.name, self.version
        )
    }
}
