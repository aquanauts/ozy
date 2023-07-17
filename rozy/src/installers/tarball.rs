use std::io::Read;

use crate::utils::download_to;

use super::installer::Installer;

use anyhow::{anyhow, Error, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use tar::Archive;
use tempfile::NamedTempFile;
use xz2::read::XzDecoder;

pub struct Tarball {
    name: String,
    version: String,
    url: String,
}

impl Tarball {
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

        Ok(Tarball {
            name: name.clone(),
            version: version.to_string(),
            url,
        })
    }
}

impl Installer for Tarball {
    fn install(&self, to_dir: &std::path::Path) -> Result<()> {
        eprintln!("Running {}", self.describe());
        std::fs::create_dir_all(to_dir)?;

        let file = NamedTempFile::new()?;
        download_to(&file.path().to_path_buf(), &self.url)?;

        let downloaded_file = std::fs::File::open(file.path())?;
        let reader: Box<dyn Read> = match infer::get_from_path(file.path())? {
            Some(t) => match t.extension() {
                "gz" => Box::new(GzDecoder::new(downloaded_file)),
                "bz2" => Box::new(BzDecoder::new(downloaded_file)),
                "xz" => Box::new(XzDecoder::new(downloaded_file)),
                _ => return Err(anyhow!("unsupported archive compression type {}", t)),
            },
            _ => {
                return Err(anyhow!(
                    "unable to determine archive compression type from file: {}",
                    file.path().display()
                ))
            }
        };

        let mut archive = Archive::new(reader);
        archive.unpack(to_dir)?;

        Ok(())
    }

    fn describe(&self) -> String {
        format!("tarball installer for {} v.{}", self.name, self.version)
    }
}
