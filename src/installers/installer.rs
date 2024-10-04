use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};

pub trait Installer {
    fn install(&self, to_dir: &std::path::Path) -> Result<()>;
    fn describe(&self) -> String;
}

pub fn run_subcommand_for_installer<'a>(
    command: &str,
    args: impl Iterator<Item = &'a str>,
    env: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    let mut command = Command::new(command);
    // Directing /dev/null to stdin breaks Conda's progress bar, for some reason
    // command.stdin(Stdio::null())
    command.stderr(Stdio::inherit());
    command.stdout(os_pipe::dup_stderr()?);

    for (env_key, env_val) in env {
        command.env(env_key, env_val);
    }

    for arg in args {
        command.arg(arg);
    }

    let mut output = command.spawn().unwrap();
    if !output.wait()?.success() {
        return Err(anyhow!(
            "Installer subcommand {:?} exited with error",
            command
        ));
    }

    Ok(())
}
