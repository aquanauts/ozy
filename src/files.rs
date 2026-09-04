use std::env;
use std::path::Path;

use anyhow::{Context, Error, Result};
use file_lock::{FileLock, FileOptions};

fn get_home_dir() -> Result<std::path::PathBuf, Error> {
    let home_dir = std::env::var("HOME").context("While checking $HOME")?;
    Ok(std::path::PathBuf::from(home_dir))
}

pub fn delete_if_exists(path: &std::path::Path) -> Result<()> {
    if path.is_symlink() {
        std::fs::remove_file(path)?
    } else if path.is_dir() {
        std::fs::remove_dir_all(path)?
    } else if path.exists() {
        std::fs::remove_file(path)?
    }

    Ok(())
}

pub fn get_ozy_dir() -> Result<std::path::PathBuf, Error> {
    Ok(get_home_dir()?.join(".ozy"))
}

pub fn get_ozy_bin_dir() -> Result<std::path::PathBuf, Error> {
    Ok(get_ozy_dir()?.join("bin"))
}

pub fn get_ozy_cache_dir() -> Result<std::path::PathBuf, Error> {
    Ok(get_home_dir()?.join(".cache").join("ozy"))
}

pub fn ensure_ozy_dirs() -> Result<(), Error> {
    std::fs::create_dir_all(get_ozy_dir()?.as_path()).context("While checking Ozy dir")?;
    std::fs::create_dir_all(get_ozy_bin_dir()?.as_path()).context("While checking Ozy bin dir")?;
    std::fs::create_dir_all(get_ozy_cache_dir()?.as_path())
        .context("While checking Ozy cache dir")?;
    Ok(())
}

pub fn delete_ozy_dirs() -> Result<(), Error> {
    delete_if_exists(get_ozy_dir()?.as_path()).context("While deleting Ozy dir")?;
    delete_if_exists(get_ozy_bin_dir()?.as_path()).context("While deleting Ozy bin dir")?;
    delete_if_exists(get_ozy_cache_dir()?.as_path()).context("While deleting Ozy cache dir")?;
    Ok(())
}

pub fn check_path(ozy_bin_dir: &std::path::Path) -> Result<bool, Error> {
    let bin_path_canonical = ozy_bin_dir.canonicalize()?;
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if let Ok(path) = path.canonicalize()
                && bin_path_canonical == path
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn softlink(from_command: &str, to_command: &str) -> Result<bool> {
    let from_command_path = get_ozy_bin_dir()?.join(from_command);
    let to_command_path = get_ozy_bin_dir()?.join(to_command);
    let was_there = from_command_path.as_path().exists();
    if was_there {
        std::fs::remove_dir_all(&from_command_path)?;
        // TODO: unlink from_command_path? Also is this equivalent to what's happening in Python where we unlink path_to_app?
    }

    std::os::unix::fs::symlink(&to_command_path, &from_command_path).with_context(|| {
        format!(
            "While symlinking {} to {}",
            from_command_path.display(),
            to_command_path.display()
        )
    })?;
    Ok(was_there)
}

fn get_trash_dir(cache_dir: &Path) -> std::path::PathBuf {
    cache_dir.join(".trash")
}

/// The installed versions of an app, taken from the `<version>.lock` files in its cache directory.
/// Lock files are the only reliable marker: the version itself may be a directory, a symlink into
/// `internal_install`, or a symlink to a legacy `<version>.<uuid>` directory.
pub fn versions_in(app_base: &Path) -> Result<Vec<String>> {
    let entries = match std::fs::read_dir(app_base) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(err) => {
            return Err(err).with_context(|| format!("While reading {}", app_base.display()));
        }
    };

    let mut result = vec![];
    for entry in entries.flatten() {
        if let Some(version) = entry
            .file_name()
            .to_string_lossy()
            .strip_suffix(".lock")
            .filter(|version| !version.is_empty())
        {
            result.push(version.to_string());
        }
    }

    Ok(result)
}

/// Move `target` out of the way with a single atomic rename, then delete it. If we're interrupted
/// part way through the delete, the leftovers are in the trash rather than looking like a valid
/// install; `sweep_trash` finishes the job next time.
pub fn trash_and_remove(cache_dir: &Path, target: &Path) -> Result<()> {
    let trash_loc = get_trash_dir(cache_dir).join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(trash_loc.parent().unwrap()).context("While creating trash dir")?;
    std::fs::rename(target, &trash_loc)
        .with_context(|| format!("While moving {} to the trash", target.display()))?;
    std::fs::remove_dir_all(&trash_loc)
        .with_context(|| format!("While deleting {}", trash_loc.display()))
}

/// Delete anything left behind by an interrupted `trash_and_remove`.
pub fn sweep_trash(cache_dir: &Path) -> Result<()> {
    let trash_dir = get_trash_dir(cache_dir);
    for entry in std::fs::read_dir(&trash_dir)
        .into_iter()
        .flatten()
        .flatten()
    {
        // best effort; a failure here is someone else's live delete or a permissions
        // problem, and either way the next sweep can retry.
        let _ = delete_if_exists(&entry.path());
    }
    Ok(())
}

/// Remove the installed tree for `version` of the app rooted at `app_base`. Handles all three
/// layouts ozy has used: a plain directory, a symlink into `internal_install`, and a symlink to a
/// legacy `<version>.<uuid>` sibling. Does not touch the lock file.
pub fn remove_installed_version(cache_dir: &Path, app_base: &Path, version: &str) -> Result<()> {
    let installed_path = app_base.join(version);

    let tree = if installed_path.is_symlink() {
        let target = installed_path
            .read_link()
            .with_context(|| format!("While reading link {}", installed_path.display()))?;
        std::fs::remove_file(&installed_path)
            .with_context(|| format!("While removing link {}", installed_path.display()))?;
        // Only delete what we installed; a symlink pointing outside the cache isn't ours.
        (target.starts_with(cache_dir) && target.is_dir()).then_some(target)
    } else if installed_path.is_dir() {
        Some(installed_path)
    } else {
        None
    };

    match tree {
        Some(tree) => trash_and_remove(cache_dir, &tree),
        None => Ok(()),
    }
}

pub fn lock_ozy_dir() -> Result<FileLock> {
    let lock_for_writing = FileOptions::new().create(true).write(true).read(true);
    let lock_path = get_ozy_dir()?.join("ozy.lock");
    let lock = FileLock::lock(lock_path, true, lock_for_writing).context("Locking ozy config")?;
    Ok(lock)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lay out all three historical install shapes for one app, plus a symlink to somewhere we
    /// don't own, and check that we find and remove exactly the right things.
    #[test]
    fn versions_and_removal_across_layouts() {
        let cache = tempfile::tempdir().expect("tempdir");
        let cache_dir = cache.path();
        let app_base = cache_dir.join("meter");
        let internal = cache_dir.join("internal_install").join("meter");
        let elsewhere = tempfile::tempdir().expect("tempdir");
        let outside = elsewhere.path().join("not-ours");

        // 1.0: a plain directory
        std::fs::create_dir_all(app_base.join("1.0/bin")).unwrap();
        // 2.0: a symlink into internal_install
        std::fs::create_dir_all(internal.join("2.0/bin")).unwrap();
        std::os::unix::fs::symlink(internal.join("2.0"), app_base.join("2.0")).unwrap();
        // 3.0: a legacy symlink to a <version>.<uuid> sibling
        std::fs::create_dir_all(app_base.join("3.0.abc-123/bin")).unwrap();
        std::os::unix::fs::symlink(app_base.join("3.0.abc-123"), app_base.join("3.0")).unwrap();
        // 4.0: a symlink to a directory outside the cache - not ours to delete
        std::fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, app_base.join("4.0")).unwrap();

        for version in ["1.0", "2.0", "3.0", "4.0"] {
            std::fs::write(app_base.join(format!("{}.lock", version)), "").unwrap();
        }

        let mut versions = versions_in(&app_base).unwrap();
        versions.sort();
        assert_eq!(versions, ["1.0", "2.0", "3.0", "4.0"]);
        assert!(
            versions_in(&cache_dir.join("nonexistent"))
                .unwrap()
                .is_empty()
        );

        for version in ["1.0", "2.0", "3.0", "4.0"] {
            remove_installed_version(cache_dir, &app_base, version).unwrap();
        }

        assert!(!app_base.join("1.0").exists());
        assert!(!internal.join("2.0").exists());
        assert!(!app_base.join("3.0.abc-123").exists());
        assert!(!app_base.join("3.0").is_symlink());
        assert!(outside.is_dir(), "must not delete outside the cache");
        // Removing a version that isn't installed is a no-op, not an error.
        remove_installed_version(cache_dir, &app_base, "9.9").unwrap();

        // Nothing lingers in the trash, and an interrupted delete gets swept up.
        let leftover = cache_dir.join(".trash").join("interrupted");
        std::fs::create_dir_all(leftover.join("bin")).unwrap();
        sweep_trash(cache_dir).unwrap();
        assert_eq!(
            std::fs::read_dir(cache_dir.join(".trash")).unwrap().count(),
            0
        );
    }
}
