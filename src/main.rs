use std::{path, time::Duration};

use anyhow::{anyhow, Context, Error, Result};
use clap::{Parser, Subcommand};
use semver::Version;
use std::os::unix::fs::PermissionsExt;
use uuid::Uuid;
use which::which;

use crate::files::{check_path, get_ozy_bin_dir};

mod app;
mod config;
mod files;
mod installers;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn symlink_binaries(path_to_ozy: &std::path::PathBuf, config: &serde_yaml::Mapping) -> Result<()> {
    // If this binary isn't installed in the correct location, move it there
    let expected_path_to_ozy = files::get_ozy_bin_dir()?.join("ozy");
    if path_to_ozy != &expected_path_to_ozy {
        // attempt to rename (same filesystem), but fall back to copy and remove (different filesystem)
        match std::fs::rename(path_to_ozy, expected_path_to_ozy.clone()) {
            Ok(_) => {},
            Err(_) => {
                std::fs::copy(path_to_ozy, expected_path_to_ozy)?;
                std::fs::remove_file(path_to_ozy)?;
            },
        }
    }

    let app_configs = match config.get("apps") {
        Some(serde_yaml::Value::Mapping(app_configs)) => app_configs,
        _ => {
            return Err(anyhow!("Expected an mapping-type apps section in the YAML",));
        }
    };

    for (name, _) in app_configs {
        files::softlink(name.as_str().unwrap(), "ozy")?;
    }

    Ok(())
}

fn init(path_to_ozy: &std::path::PathBuf, url: &str) -> Result<()> {
    files::ensure_ozy_dirs()?;

    let mut user_conf = serde_yaml::Mapping::new();
    user_conf.insert(
        serde_yaml::Value::String("url".to_string()),
        serde_yaml::Value::String(url.to_string()),
    );
    config::save_ozy_user_conf(&user_conf)?;

    let base_ozy_conf = files::get_ozy_dir()?.join("ozy.yaml");
    utils::download_to(&base_ozy_conf, url)?;

    let config = config::load_config(None)?;
    symlink_binaries(path_to_ozy, &config)?;
    Ok(())
}

fn install(app_names: &[String]) -> Result<()> {
    files::ensure_ozy_dirs()?;
    let config = config::load_config(None)?;

    for app_name in app_names.iter() {
        let app = app::App::new(app_name, &config)?;
        app.ensure_installed()?;
    }

    Ok(())
}

fn install_all() -> Result<()> {
    files::ensure_ozy_dirs()?;
    let config = config::load_config(None)?;
    let app_configs = match config.get("apps") {
        Some(serde_yaml::Value::Mapping(app_configs)) => app_configs,
        _ => {
            return Err(anyhow!("Expected an mapping-type apps section in the YAML",));
        }
    };

    for (name, _) in app_configs {
        let name = match name {
            serde_yaml::Value::String(name) => name,
            _ => {
                return Err(anyhow!("Expected name of app config to be a string"));
            }
        };

        let app = app::App::new(name, &config);
        if app.is_err() {
            eprintln!(
                "Skipping incompatible app config for {} due to: {}",
                name,
                app.err().unwrap()
            );
            continue;
        }

        eprintln!("Installing {}", name);
        app?.ensure_installed()
            .with_context(|| format!("While ensuring app {} is installed", name))?;
    }

    Ok(())
}

fn makefile_config_internal(
    makefile_var: &String,
    app_names: &[String],
    did_path_contain_ozy: bool,
) -> Result<String> {
    if !did_path_contain_ozy {
        return Err(anyhow!("The Ozy bin directory must be in the PATH"));
    }

    let config = config::load_config(None)?;

    let ozy_bin_dir = get_ozy_bin_dir()?;
    for app_name in app_names.iter() {
        if app::App::new(app_name, &config).is_err() {
            return Err(anyhow!("Missing ozy app '{}'", app_name));
        }
        let app_in_bin = ozy_bin_dir.join(path::Path::new(app_name)).canonicalize()?;
        if let Ok(os_found) = which(app_name) {
            if os_found.canonicalize()? != app_in_bin {
                return Err(anyhow!(
                    "'{}' found in PATH earlier than ozy: results could be inconsistent (found at {})",
                    app_name,
                    os_found.display()));
            }
        } else {
            return Err(anyhow!(
                "Missing ozy app '{}' - not found on PATH",
                app_name
            ));
        }
    }
    Ok(format!("{}:={}", &makefile_var, ozy_bin_dir.display()))
}

fn makefile_config(
    makefile_var: &String,
    app_names: &[String],
    did_path_contain_ozy: bool,
) -> Result<()> {
    match makefile_config_internal(makefile_var, app_names, did_path_contain_ozy) {
        Ok(str) => {
            println!("{}", str);
        }
        Err(err) => {
            println!("$(error \"{}\")", err); // todo escape
        }
    }
    Ok(())
}

fn clean() -> Result<()> {
    files::delete_ozy_dirs()
}

fn should_update(config: &serde_yaml::Mapping) -> Result<bool> {
    match config.get("ozy_update_every") {
        Some(serde_yaml::Value::Number(update_every)) => {
            let update_every = Duration::from_secs(update_every.as_u64().unwrap_or_default());
            let since_last_update = std::time::SystemTime::now().duration_since(
                config::config_mtime().context("While determining config mtime")?,
            )?;
            Ok(!update_every.is_zero() && since_last_update >= update_every)
        }
        _ => Ok(false),
    }
}

fn run(app_name: &String, args: &[String]) -> Result<()> {
    let mut config = config::load_config(None).context("While loading ozy config")?;
    if should_update(&config)? {
        update(&std::env::current_exe()?, &None).unwrap();
        config = config::load_config(None).context("While reloading ozy config")?;
    }

    let app = app::find_app(&config, app_name)?;
    app.ensure_installed()
        .with_context(|| format!("While ensuring that app {} is installed", app_name))?;

    // TODO: Maybe restore environment variables if we have to override any
    let program_path = app.get_absolute_executable_path()?;
    let mut command = exec::Command::new(&program_path);
    for arg in args {
        command.arg(arg);
    }

    // We use exec to replace this process with the child entirely. The only time exec() returns is
    // if there was an error.
    let err = command.exec();
    Err(anyhow!(
        "Failed to execute process {} ({})",
        program_path.display(),
        err
    ))
}

fn get_apps(config: &serde_yaml::Mapping) -> Result<Vec<app::App>> {
    let app_configs = match config.get("apps") {
        Some(serde_yaml::Value::Mapping(app_configs)) => app_configs,
        _ => {
            return Err(anyhow!("Expected an mapping-type apps section in the YAML",));
        }
    };

    let mut result = vec![];
    for (name, _) in app_configs {
        let name = match name {
            serde_yaml::Value::String(name) => name,
            _ => {
                return Err(anyhow!("Expected name of app config to be a string"));
            }
        };

        let app = app::App::new(name, config);
        if app.is_err() {
            eprintln!(
                "Skipping incompatible app config for {} due to: {}",
                name,
                app.err().unwrap()
            );
            continue;
        }

        result.push(app?);
    }

    Ok(result)
}

fn update(path_to_ozy: &std::path::PathBuf, url: &Option<String>) -> Result<()> {
    files::ensure_ozy_dirs()?;
    let _lock = files::lock_ozy_dir()?;
    let old_base_ozy_conf = files::get_ozy_dir()?.join("ozy.yaml");
    let new_base_ozy_conf = files::get_ozy_dir()?.join("ozy.yaml.tmp");

    let user_config = config::get_ozy_user_conf()?;
    let url = match url {
        Some(v) => v,
        None => user_config["url"].as_str().unwrap(),
    };

    utils::download_to(&new_base_ozy_conf, url)?;

    let new_config = config::load_config(Some("ozy.yaml.tmp"))?;

    let new_version = Version::parse(new_config["ozy_version"].as_str().unwrap())?;
    if new_version > Version::parse(VERSION)? {
        eprintln!(
            "Ozy update to {} is mandated by your team config",
            new_version
        );

        let mut config_update_slice = serde_yaml::Mapping::new();
        config_update_slice.insert(
            serde_yaml::Value::String("ozy_download".to_string()),
            new_config["ozy_download"].clone(),
        );
        config_update_slice.insert(
            serde_yaml::Value::String("version".to_string()),
            new_config["ozy_version"].clone(),
        );
        config::resolve(&mut config_update_slice);
        let download_url = config_update_slice["ozy_download"].as_str().unwrap();
        eprintln!("Downloading from {}", download_url);

        let dest_path = files::get_ozy_bin_dir()?.join(format!("ozy.tmp.{}", Uuid::new_v4()));
        files::delete_if_exists(&dest_path)?;
        utils::download_to(&dest_path, download_url)?;
        let mut perms = std::fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_path, perms)?;

        let err = exec::execvp(&dest_path, std::env::args().collect::<Vec<String>>());
        return Err(anyhow!(
            "Failed to execute process {} ({})",
            dest_path.display(),
            err
        ));
    }

    std::fs::rename(new_base_ozy_conf, old_base_ozy_conf)?;
    let config = config::load_config(None)?;
    symlink_binaries(path_to_ozy, &config)?;

    let mut user_conf = serde_yaml::Mapping::new();
    user_conf.insert(
        serde_yaml::Value::String("url".to_string()),
        serde_yaml::Value::String(url.to_string()),
    );
    config::save_ozy_user_conf(&user_conf)?;

    Ok(())
}

fn sync(path_to_ozy: &std::path::PathBuf) -> Result<()> {
    let config = config::load_config(None)?;
    symlink_binaries(path_to_ozy, &config)?;

    Ok(())
}

fn list() -> Result<()> {
    let config = config::load_config(None)?;
    let apps = get_apps(&config)?;
    for app in apps.iter() {
        println!("{}", app.name);
    }

    Ok(())
}

fn show_path_warning() -> Result<()> {
    let ozy_bin_dir = files::get_ozy_bin_dir()?.to_str().unwrap().to_owned();

    println!("{:-<1$}", "", 80);
    println!("Please ensure '{}' is on your path", ozy_bin_dir);
    println!("bash shell users:");
    println!(
        "  bash$ echo -e '# ozy support\\nexport PATH={}:$PATH' >> ~/.bashrc",
        ozy_bin_dir
    );
    println!("  then restart your shell sessions");
    println!("zsh shell users:");
    println!("  zsh$ # path+=({})\\nexport PATH", ozy_bin_dir);
    println!("fish shell users: ");
    println!(
        "  fish$ set --universal fish_user_paths {} $fish_user_paths",
        ozy_bin_dir
    );
    println!("{:-<1$}", "", 80);

    Ok(())
}

fn info(did_path_contain_ozy: bool) -> Result<()> {
    if !did_path_contain_ozy {
        show_path_warning()?;
    }

    let user_config = config::get_ozy_user_conf()?;
    let team_url = match user_config.get("url") {
        Some(v) => v.as_str().unwrap(),
        None => "(unset)",
    };
    println!("Team URL: {}", team_url);

    let config = config::load_config(None)?;
    let team_config_name = match config.get("name") {
        Some(v) => v.as_str().unwrap(),
        None => "(unset)",
    };
    println!("Team config name: {}", team_config_name);

    let apps = get_apps(&config)?;
    for app in apps.iter() {
        println!("  {}", app);
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Cleans up the ozy-controlled directory")]
    Clean,

    #[clap(
        trailing_var_arg = true,
        about = "Ensures the named applications are installed at their current prevailing versions"
    )]
    Install { app_names: Vec<String> },

    #[clap(
        trailing_var_arg = true,
        about = "Checks apps, and prints a single-line Makefile variable",
        long_about = r#"
Use as an argument to $(eval). Errors will are output as $(error) directives
to report in make.
The given variable is defined to be the ozy binary directory, so any app will be
$(VAR)/app_name. If undefined, you know ozy isn't installed.

Example:

$ cat Makefile
$(eval $(shell ozy makefile-config OZY_BIN_DIR terraform))
ifndef OZY_BIN_DIR
$(error please install ozy)
endif

install:
    $(OZY_BIN_DIR)/terraform apply
"#
    )]
    MakefileConfig {
        makefile_var: String,
        app_names: Vec<String>,
    },

    #[clap(about = "Ensures all applications are installed at their current prevailing versions")]
    InstallAll,

    #[clap(about = "Initialise and install ozy, with configuration from the given URL")]
    Init { url: String },

    #[clap(about = "Print information about the installation and configuration")]
    Info,

    #[clap(about = "List all the managed apps")]
    List,

    #[clap(trailing_var_arg = true, about = "Runs the given application")]
    Run {
        app_name: String,
        app_args: Vec<String>,
    },

    #[clap(about = "Update base configuration from the remote URL")]
    Update {
        #[arg(short, long)]
        url: Option<String>,
    },

    #[clap(
        about = "Synchronise any local changes",
        long_about = r#"
If you're defining new applications in local user files, you can use this to ensure
the relevant symlinks are created in your ozy bin directory.
"#
    )]
    Sync,
}

fn main() -> Result<(), Error> {
    files::ensure_ozy_dirs()?;
    let ozy_bin_dir = get_ozy_bin_dir()?;
    let did_path_contain_ozy = check_path(&ozy_bin_dir)?;
    if !did_path_contain_ozy {
        let updated_path = format!("{}:{}", ozy_bin_dir.display(), std::env::var("PATH")?);
        std::env::set_var("PATH", updated_path);
    }

    let invoked_as = std::env::args()
        .next()
        .as_ref()
        .map(std::path::Path::new)
        .and_then(std::path::Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from)
        .unwrap();

    if invoked_as.starts_with("ozy") {
        let args = Args::parse();
        let exe_path = std::env::current_exe()?;
        match &args.command {
            Commands::Clean => clean(),
            Commands::Init { url } => init(&exe_path, url),
            Commands::Install { app_names } => install(app_names),
            Commands::InstallAll => install_all(),
            Commands::Info => info(did_path_contain_ozy),
            Commands::List => list(),
            Commands::MakefileConfig {
                makefile_var,
                app_names,
            } => makefile_config(makefile_var, app_names, did_path_contain_ozy),
            Commands::Run { app_name, app_args } => run(app_name, app_args),
            Commands::Update { url } => update(&exe_path, url),
            Commands::Sync => sync(&exe_path),
        }
    } else {
        let args = std::env::args().collect::<Vec<String>>();
        run(&invoked_as, &args[1..])
    }
}
