use anyhow::{Context, Error, Result, anyhow};
use file_lock::{FileLock, FileOptions};

use crate::config::{apply_overrides, resolve};
use crate::files::{
    delete_if_exists, get_ozy_cache_dir, remove_installed_version, trash_and_remove, versions_in,
};
use crate::installers::conda::Conda;
use crate::installers::file::File;
use crate::installers::installer::Installer;
use crate::installers::pip::Pip;
use crate::installers::shell::Shell;
use crate::installers::single_binary_zip::SingleBinaryZip;
use crate::installers::symlink::Symlink;
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
    Symlink,
}

pub struct App {
    pub name: String,
    pub version: String,
    installer: Box<dyn Installer>,
    relocatable: bool,
    executable_path: String,
}

pub fn find_app(config: &serde_yaml_ng::Mapping, app: &String) -> Result<App> {
    App::new(app, config)
        .with_context(|| format!("While attempting to find the app {} to run", app))
}

impl App {
    pub fn new(name: &String, config: &serde_yaml_ng::Mapping) -> Result<Self, Error> {
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

        if let Some(serde_yaml_ng::Value::String(template)) = app_config.get("template") {
            let mut new_app_config = config["templates"][template].clone();
            apply_overrides(&app_config, new_app_config.as_mapping_mut().unwrap());
            app_config = new_app_config.as_mapping().unwrap().clone();
        }

        resolve(&mut app_config);
        let version = match app_config.get("version") {
            Some(serde_yaml_ng::Value::String(version)) => version.clone(),
            _ => {
                return Err(anyhow!("Expected a string version in config for {}", name));
            }
        };

        let relocatable = match app_config.get("relocatable") {
            Some(serde_yaml_ng::Value::Bool(value)) => value.to_owned(),
            _ => true,
        };

        let app_type = match app_config.get("type") {
            Some(serde_yaml_ng::Value::String(app_type)) => match &app_type[..] {
                "single_binary_zip" => AppType::SingleBinaryZip,
                "tarball" => AppType::Tarball,
                "shell_install" => AppType::Shell,
                "single_file" => AppType::File,
                "pip" => AppType::Pip,
                "conda" => AppType::Conda,
                "zip" => AppType::Zip,
                "symlink" => AppType::Symlink,
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
            AppType::Symlink => Box::new(Symlink::new(name, &version, &app_config)?),
        };

        let executable_path = match app_config.get("executable_path") {
            Some(serde_yaml_ng::Value::String(value)) => value,
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
        if self.is_installed().context("Checking if it's installed")? {
            return Ok(());
        }
        let install_dir = self.get_install_path()?;
        std::fs::create_dir_all(install_dir.parent().unwrap())
            .context("While creating parent directory of install directory")?;

        let _lock = self.lock_version(&self.version)?;
        // Someone else may have installed it while we were waiting for the lock.
        if self.is_installed().context("Checking if it's installed")? {
            return Ok(());
        }

        // Nothing valid is installed, and we hold the lock, so anything sitting at install_dir
        // is junk from an earlier run - a dangling symlink, a stray file. Clear it, or the
        // rename/symlink below fails with ENOTDIR/EEXIST on every future run.
        delete_if_exists(install_dir.as_path())?;

        // In Python we generate a UUID here; is that necessary?
        let uniq_install_dir = self
            .get_internal_install_path()
            .context("Checking its install path")?;
        delete_if_exists(uniq_install_dir.as_path())?;

        let result = self.install_and_link(&uniq_install_dir, &install_dir);
        if result.is_err() {
            // Tidy up our own half-finished work, and only that: anything already installed
            // belongs to whoever successfully put it there. We still hold the lock, so nobody
            // else is using this directory.
            let _ = delete_if_exists(uniq_install_dir.as_path());
        }

        result.with_context(|| format!("While installing {} v{}", self.name, self.version))
    }

    /// Run the installer into `uniq_install_dir`, then put it in place at `install_dir` - by
    /// renaming for a relocatable app, and by symlinking for one that has to stay put.
    fn install_and_link(
        &self,
        uniq_install_dir: &std::path::Path,
        install_dir: &std::path::Path,
    ) -> Result<()> {
        self.installer
            .install(uniq_install_dir)
            .context("While running the Installer")?;

        let put_in_place = match self.relocatable {
            true => std::fs::rename(uniq_install_dir, install_dir).context("Renaming"),
            false => {
                std::os::unix::fs::symlink(uniq_install_dir, install_dir).context("Symlinking")
            }
        };

        // if put_in_place contains Err, it means another process (probably a parallel ozy)
        // installed the package we were working on. Trash our work-in-progress one since theirs
        // completed successfully; if the app is not relocatable then do nothing, since the
        // equivalent install is completed.
        match put_in_place {
            Err(err) if !self.is_installed().unwrap_or(false) => Err(err),
            Err(_) => {
                if self.relocatable {
                    // Best-effort: the app is installed, so failing to bin our redundant copy
                    // is not worth failing the user's command over.
                    let _ = trash_and_remove(&get_ozy_cache_dir()?, uniq_install_dir);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn lock_version(&self, version: &str) -> Result<FileLock> {
        let lock_for_writing = FileOptions::new().create(true).write(true).read(true);
        let lockfile_path = self.get_app_base()?.join(format!("{}.lock", version));
        let _lock =
            FileLock::lock(lockfile_path, true, lock_for_writing).context("Locking file")?;

        Ok(_lock)
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

    fn get_app_base(&self) -> Result<std::path::PathBuf, Error> {
        Ok(get_ozy_cache_dir()?.join(&self.name))
    }

    pub fn versions(&self) -> Result<Vec<String>> {
        versions_in(&self.get_app_base()?)
    }

    pub fn uninstall_version(&self, installed_version: &str) -> Result<()> {
        let lock = self.lock_version(installed_version)?;
        let app_base = self.get_app_base()?;

        remove_installed_version(&get_ozy_cache_dir()?, &app_base, installed_version)
            .with_context(|| format!("While deleting {}@{}", self.name, installed_version))?;

        std::fs::remove_file(app_base.join(format!("{}.lock", installed_version)))
            .context("While removing lock file")?;

        lock.unlock()?;

        Ok(())
    }

    pub fn prune_other_versions(&self, dry_run: bool) -> Result<()> {
        for installed_version in self.versions()? {
            if installed_version != self.version {
                if dry_run {
                    println!("Would prune {} {}", self.name, installed_version);
                } else {
                    println!("Pruning {} {}", self.name, installed_version);
                    self.uninstall_version(&installed_version)?;
                }
            }
        }
        Ok(())
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
    use serde_yaml_ng::Mapping;

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
        assert!(app.relocatable);
        assert_eq!(app.executable_path, "bin/single_binary_zip_app");
    }

    #[test]
    fn reloctable_detection_test() {
        let config = get_test_config();
        let app = App::new(&"explicitly_relocatable_app".to_string(), &config)
            .expect("Failed to construct App");
        assert!(app.relocatable);

        let app = App::new(&"explicitly_nonrelocatable_app".to_string(), &config)
            .expect("Failed to construct App");
        assert!(!app.relocatable);

        let app = App::new(&"unspecified_relocatability_app".to_string(), &config)
            .expect("Failed to construct App");
        assert!(app.relocatable);
    }

    #[test]
    fn single_binary_zip_test() {
        let config = get_test_config();
        let single_binary_zip_app = App::new(&"single_binary_zip_app".to_string(), &config)
            .expect("Failed to construct App");
        assert_eq!(
            single_binary_zip_app.installer.describe(),
            "single binary zip installer for single_binary_zip_app v1.10.1"
        );
    }

    #[test]
    fn tarball_test() {
        let config = get_test_config();
        let tarball_app =
            App::new(&"tarball_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            tarball_app.installer.describe(),
            "tarball installer for tarball_app v0.7.0"
        );
    }

    #[test]
    fn shell_test() {
        let config = get_test_config();
        let shell_app =
            App::new(&"shell_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            shell_app.installer.describe(),
            "shell installer for shell_app v4.11.0"
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
    fn pip_pinned_test() {
        let config = get_test_config();
        let pip_app =
            App::new(&"pip_app_pinned".to_string(), &config).expect("Failed to construct App");
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
            "file installer for file_app v6.6.0"
        );
    }
    #[test]
    fn symlink_test() {
        let config = get_test_config();
        let file_app =
            App::new(&"symlink_app".to_string(), &config).expect("Failed to construct App");
        assert_eq!(
            file_app.installer.describe(),
            "symlink installer for symlink_app v5.6.7"
        );
    }

    /// A fake installer that leaves a marker behind. `delay` widens the window in which two
    /// racing processes are both mid-install.
    struct FakeInstaller {
        delay: std::time::Duration,
    }

    impl Installer for FakeInstaller {
        fn install(&self, to_dir: &std::path::Path) -> Result<()> {
            std::fs::create_dir_all(to_dir)?;
            std::fs::write(to_dir.join("marker"), "installed")?;
            std::thread::sleep(self.delay);
            Ok(())
        }

        fn describe(&self) -> String {
            "fake installer".to_string()
        }
    }

    /// A fake installer that half-populates the directory and then gives up.
    struct FailingInstaller;

    impl Installer for FailingInstaller {
        fn install(&self, to_dir: &std::path::Path) -> Result<()> {
            std::fs::create_dir_all(to_dir)?;
            std::fs::write(to_dir.join("half-written"), "junk")?;
            Err(anyhow!("installer blew up"))
        }

        fn describe(&self) -> String {
            "failing installer".to_string()
        }
    }

    fn fake_app(name: &str, version: &str, installer: Box<dyn Installer>) -> App {
        App {
            name: name.to_string(),
            version: version.to_string(),
            installer,
            relocatable: true,
            executable_path: format!("bin/{}", name),
        }
    }

    /// Point ozy's cache at `home`. $HOME is process-global, so tests that call this have to
    /// hold `HOME_LOCK` for as long as they use it.
    fn set_home(home: &std::path::Path) {
        // SAFETY: every test that sets or reads the environment holds HOME_LOCK for the
        // duration, so no other thread is in getenv/setenv concurrently.
        unsafe { std::env::set_var("HOME", home) };
    }

    static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Take HOME_LOCK, ignoring poisoning so a panicking test doesn't mask its sibling's
    /// assertion failure with a PoisonError. Restores $HOME when dropped, so a test's tempdir
    /// doesn't outlive it as a dangling $HOME for the rest of the process.
    fn lock_home() -> HomeGuard {
        HomeGuard {
            was: std::env::var_os("HOME"),
            _lock: HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }

    struct HomeGuard {
        was: Option<std::ffi::OsString>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            // SAFETY: runs before _lock drops, so we still hold HOME_LOCK - see set_home.
            match &self.was {
                Some(home) => unsafe { std::env::set_var("HOME", home) },
                None => unsafe { std::env::remove_var("HOME") },
            }
        }
    }

    const CHILD_RAN: &str = "child-ran";

    /// The other half of `concurrent_installs_do_not_clobber`; a no-op in a normal test run.
    /// ozy locks with fcntl, which is per-process, so the race only reproduces across processes.
    #[test]
    fn concurrent_install_child() {
        let _guard = lock_home();
        if let Ok(home) = std::env::var("OZY_TEST_CHILD_HOME") {
            set_home(std::path::Path::new(&home));
            fake_app(
                "racy_app",
                "1.0",
                Box::new(FakeInstaller {
                    delay: std::time::Duration::from_millis(500),
                }),
            )
            .ensure_installed()
            .expect("child install failed");
            // Proof for the parent that this test actually ran: a filter that matches nothing
            // also exits 0, so the child's exit status alone doesn't say the race was exercised.
            std::fs::write(std::path::Path::new(&home).join(CHILD_RAN), "").expect("sentinel");
        }
    }

    /// Two processes installing the same app at once: the loser of the lock must notice the
    /// winner already installed it and leave it be, rather than reinstalling and renaming on top
    /// of it (which fails with ENOTEMPTY, and then took the whole app directory down with it).
    #[test]
    fn concurrent_installs_do_not_clobber() {
        let _guard = lock_home();
        let home = tempfile::tempdir().expect("tempdir");
        let home_str = home.path().to_str().unwrap().to_string();

        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "--nocapture",
                "app::tests::concurrent_install_child",
            ])
            .env("OZY_TEST_CHILD_HOME", &home_str)
            .spawn()
            .expect("spawning child test process");

        set_home(home.path());
        let ours = fake_app(
            "racy_app",
            "1.0",
            Box::new(FakeInstaller {
                delay: std::time::Duration::from_millis(500),
            }),
        )
        .ensure_installed();
        let theirs = child.wait().expect("waiting for child test process");

        ours.expect("install in this process failed");
        assert!(theirs.success(), "install in the child process failed");
        assert!(
            home.path().join(CHILD_RAN).is_file(),
            "the child test never ran, so nothing raced - has it been renamed?"
        );
        assert!(
            home.path().join(".cache/ozy/racy_app/1.0/marker").is_file(),
            "the installed app was clobbered by the racing install"
        );
    }

    /// A failed install must clean up after itself without taking the rest of the app's cache
    /// directory - other installed versions, and the lock files - with it.
    #[test]
    fn failed_install_leaves_the_rest_of_the_app_alone() {
        let _guard = lock_home();
        let home = tempfile::tempdir().expect("tempdir");
        let cache = home.path().join(".cache/ozy");

        set_home(home.path());
        fake_app(
            "versioned_app",
            "1.0",
            Box::new(FakeInstaller {
                delay: std::time::Duration::ZERO,
            }),
        )
        .ensure_installed()
        .expect("installing v1.0");

        let err = fake_app("versioned_app", "2.0", Box::new(FailingInstaller))
            .ensure_installed()
            .expect_err("v2.0 should have failed to install");
        assert!(
            format!("{:#}", err).contains("installer blew up"),
            "{:#}",
            err
        );

        assert!(
            cache.join("versioned_app/1.0/marker").is_file(),
            "the failed install deleted a good one"
        );
        assert!(
            cache.join("versioned_app/1.0.lock").is_file(),
            "the failed install deleted the lock files"
        );
        assert!(
            !cache.join("internal_install/versioned_app/2.0").exists(),
            "the failed install left its half-written directory behind"
        );
    }

    /// A dangling symlink at the install path - left by a non-relocatable install whose target
    /// went away - used to wedge the app forever: it isn't a valid install, so the rename onto
    /// it failed with ENOTDIR on every subsequent run.
    #[test]
    fn a_wedged_install_path_recovers() {
        let _guard = lock_home();
        let home = tempfile::tempdir().expect("tempdir");
        let install_dir = home.path().join(".cache/ozy/wedged_app/1.0");

        std::fs::create_dir_all(install_dir.parent().unwrap()).expect("cache dir");
        std::os::unix::fs::symlink(home.path().join("gone"), &install_dir).expect("symlink");

        set_home(home.path());
        fake_app(
            "wedged_app",
            "1.0",
            Box::new(FakeInstaller {
                delay: std::time::Duration::ZERO,
            }),
        )
        .ensure_installed()
        .expect("installing over a dangling symlink");

        assert!(
            install_dir.join("marker").is_file(),
            "the install did not replace the dangling symlink"
        );
    }
}
