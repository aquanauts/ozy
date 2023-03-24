use anyhow::{anyhow, Context, Error, Result};
use file_lock::{FileLock, FileOptions};

use crate::config::{apply_overrides, resolve};
use crate::files::{delete_if_exists, get_ozy_cache_dir};
use crate::installers::conda::Conda;
use crate::installers::file::File;
use crate::installers::installer::Installer;
use crate::installers::pip::Pip;
use crate::installers::shell::Shell;
use crate::installers::single_binary_zip::SingleBinaryZip;
use crate::installers::tarball::Tarball;
use crate::installers::zip::Zip;

pub enum AppType {
    SingleBinaryZip,
    Tarball,
    Shell,
    File,
    Conda,
    Pip,
    Zip,
}

pub struct App {
    pub name: String,
    pub version: String,
    installer: Box<dyn Installer>,
    relocatable: bool,
    executable_path: String,
}

pub fn find_app(config: &serde_yaml::Mapping, app: &String) -> Result<App> {
    App::new(app, &config)
        .with_context(|| format!("While attempting to find the app {} to run", app))
}

impl App {
    pub fn new(name: &String, config: &serde_yaml::Mapping) -> Result<Self, Error> {
        let app_configs = config
            .get("apps")
            .ok_or_else(|| anyhow!("Expected an apps section in the YAML"))?
            .as_mapping()
            .unwrap();

        let mut app_config = app_configs
            .get(name)
            .ok_or_else(|| anyhow!("Could not find app {} in config", name))?
            .as_mapping()
            .unwrap()
            .clone();

        if let Some(serde_yaml::Value::String(template)) = app_config.get("template") {
            let mut new_app_config = config["templates"][template].clone();
            apply_overrides(&app_config, new_app_config.as_mapping_mut().unwrap());
            app_config = new_app_config.as_mapping().unwrap().clone();
        }

        resolve(&mut app_config);
        let version = match app_config.get("version") {
            Some(serde_yaml::Value::String(version)) => version.clone(),
            _ => {
                return Err(anyhow!("Expected a string version in config for {}", name));
            }
        };

        let relocatable = match app_config.get("relocatable") {
            Some(serde_yaml::Value::Bool(value)) => value.to_owned(),
            _ => true,
        };

        let app_type = match app_config.get("type") {
            Some(serde_yaml::Value::String(app_type)) => match &app_type[..] {
                "single_binary_zip" => AppType::SingleBinaryZip,
                "tarball" => AppType::Tarball,
                "shell_install" => AppType::Shell,
                "single_file" => AppType::File,
                "pip" => AppType::Pip,
                "conda" => AppType::Conda,
                "zip" => AppType::Zip,
                _ => {
                    return Err(anyhow!("App type {} not yet supported", &app_type[..]));
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected a type field for app {} that contains a string in the YAML",
                    name
                ));
            }
        };

        let installer: Box<dyn Installer> = match app_type {
            AppType::SingleBinaryZip => {
                Box::new(SingleBinaryZip::new(name, &version, &app_config)?)
            }
            AppType::Zip => Box::new(Zip::new(name, &version, &app_config)?),
            AppType::Tarball => Box::new(Tarball::new(name, &version, &app_config)?),
            AppType::Shell => Box::new(Shell::new(name, &version, &app_config)?),
            AppType::File => Box::new(File::new(name, &version, &app_config)?),
            AppType::Conda => Box::new(Conda::new(name, &version, &app_config)?),
            AppType::Pip => Box::new(Pip::new(name, &version, &app_config)?),
        };

        let executable_path = match app_config.get("executable_path") {
            Some(serde_yaml::Value::String(value)) => value,
            _ => name,
        };

        Ok(App {
            name: name.clone(),
            version,
            installer,
            relocatable,
            executable_path: executable_path.clone(),
        })
    }

    pub fn ensure_installed(&self) -> Result<()> {
        let result = self.ensure_installed_internal();
        if result.is_err() {
            delete_if_exists(self.get_install_path()?.parent().unwrap())?;
        }

        result
    }

    fn ensure_installed_internal(&self) -> Result<()> {
        if self.is_installed().context("Checking if it's installed")? {
            return Ok(());
        }

        let install_dir = self.get_install_path()?;
        std::fs::create_dir_all(install_dir.parent().unwrap())
            .context("While creating parent directory of install directory")?;

        let lock_for_writing = FileOptions::new().create(true).write(true).read(true);
        let lockfile_path = install_dir.parent().unwrap().join(format!(
            "{}.lock",
            install_dir.file_name().unwrap().to_string_lossy()
        ));
        let _lock =
            FileLock::lock(lockfile_path, true, lock_for_writing).context("Locking file")?;
        if self.is_installed().context("Checking if it's installed")? {
            return Ok(());
        }

        // In Python we generate a UUID here; is that necessary?
        let uniq_install_dir = self
            .get_internal_install_path()
            .context("Checking its install path")?;
        delete_if_exists(uniq_install_dir.as_path())?;

        self.installer
            .install(uniq_install_dir.as_path())
            .context("While running the Installer")
            .and_then(|_| {
                match self.relocatable {
                    true => std::fs::rename(&uniq_install_dir, &install_dir).context("Renaming")?,
                    false => std::os::unix::fs::symlink(&uniq_install_dir, &install_dir)
                        .context("Symlinking")?,
                };
                Ok(())
            })
            .with_context(|| format!("While installing {} v.{}", &self.name, &self.version))?;

        Ok(())
    }

    pub fn get_absolute_executable_path(&self) -> Result<std::path::PathBuf> {
        Ok(self.get_install_path()?.join(self.executable_path.clone()))
    }

    fn is_installed(&self) -> Result<bool, Error> {
        match std::fs::metadata(self.get_install_path()?) {
            Ok(metadata) => Ok(metadata.is_dir()),
            Err(_) => Ok(false),
        }
    }

    fn get_install_path(&self) -> Result<std::path::PathBuf, Error> {
        Ok(get_ozy_cache_dir()?.join(&self.name).join(&self.version))
    }

    fn get_internal_install_path(&self) -> Result<std::path::PathBuf, Error> {
        Ok(get_ozy_cache_dir()?
            .join("internal_install")
            .join(&self.name)
            .join(&self.version))
    }
}

impl std::hash::Hash for App {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.hash(state);
        self.relocatable.hash(state);
        self.executable_path.hash(state);
        self.installer.describe().hash(state);
    }
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.version == other.version
            && self.relocatable == other.relocatable
            && self.executable_path == other.executable_path
            && self.installer.describe() == other.installer.describe()
    }
}

impl Eq for App {}

impl std::fmt::Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}: {}",
            self.name,
            self.version,
            self.installer.describe()
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::Mapping;

    use crate::config::parse_ozy_config;

    use super::*;

    fn get_test_config() -> Mapping {
        let test_yaml_path = std::path::Path::new("test_resource/unittest.ozy.yaml").to_path_buf();
        parse_ozy_config(&test_yaml_path).expect("Failed to load YAML")
    }

    #[test]
    fn general_constructor_test() {
        let config = get_test_config();
        let app = App::new(&"single_binary_zip_app".to_string(), &config)
            .expect("Failed to construct App");
        assert_eq!(app.name, "single_binary_zip_app");
        assert_eq!(app.version, "1.10.1");
        assert_eq!(app.relocatable, true);
        assert_eq!(app.executable_path, "bin/single_binary_zip_app");
    }

    #[test]
    fn reloctable_detection_test() {
        let config = get_test_config();
        let app = App::new(&"explicitly_relocatable_app".to_string(), &config)
            .expect("Failed to construct App");
        assert_eq!(app.relocatable, true);

        let app = App::new(&"explicitly_nonrelocatable_app".to_string(), &config)
            .expect("Failed to construct App");
        assert_eq!(app.relocatable, false);

        let app = App::new(&"unspecified_relocatability_app".to_string(), &config)
            .expect("Failed to construct App");
        assert_eq!(app.relocatable, true);
    }

    #[test]
    fn single_binary_zip_test() {
        let config = get_test_config();
        let single_binary_zip_app = App::new(&"single_binary_zip_app".to_string(), &config)
            .expect("Failed to construct App");
        assert_eq!(
            single_binary_zip_app.installer.describe(),
            "single binary zip installer for single_binary_zip_app v.1.10.1"
        );
    }

    #[test]
    fn tarball_test() {
        let config = get_test_config();
        let tarball_app =
            App::new(&"tarball_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            tarball_app.installer.describe(),
            "tarball installer for tarball_app v.0.7.0"
        );
    }

    #[test]
    fn shell_test() {
        let config = get_test_config();
        let shell_app =
            App::new(&"shell_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            shell_app.installer.describe(),
            "shell installer for shell_app v.4.11.0"
        );
    }

    #[test]
    fn pip_test() {
        let config = get_test_config();
        let pip_app = App::new(&"pip_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            pip_app.installer.describe(),
            "pip installer for pip_package=1.20.4"
        );
    }

    #[test]
    fn conda_test() {
        let config = get_test_config();
        let conda_app =
            App::new(&"conda_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            conda_app.installer.describe(),
            "conda installer for conda-package=45"
        );
    }

    #[test]
    fn file_test() {
        let config = get_test_config();
        let file_app = App::new(&"file_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            file_app.installer.describe(),
            "file installer for file_app v.6.6.0"
        );
    }
}
