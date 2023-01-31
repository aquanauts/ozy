use std::env;

use anyhow::{Context, Error, Result};

fn get_home_dir() -> Result<std::path::PathBuf, Error> {
    let home_dir = std::env::var("HOME").context("While checking $HOME")?;
    Ok(std::path::PathBuf::from(home_dir))
}

pub fn delete_if_exists(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            std::fs::remove_dir_all(path)?
        } else {
            std::fs::remove_file(path)?
        }
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
            if let Ok(path) = path.canonicalize() {
                if bin_path_canonical == path {
                    return Ok(true);
                }
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

    std::os::unix::fs::symlink(to_command_path, from_command_path)?;
    Ok(was_there)
}
