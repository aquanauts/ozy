use anyhow::Result;

pub fn download_to(dest_path: &std::path::PathBuf, url: &str) -> Result<()> {
    let tmp_dest_path = dest_path.clone().with_extension("tmp");
    let mut dest_file = std::fs::File::create(&tmp_dest_path)?;
    let response = reqwest::blocking::get(url)?;
    let mut content = std::io::Cursor::new(response.bytes()?);
    std::io::copy(&mut content, &mut dest_file)?;
    std::fs::rename(tmp_dest_path, dest_path)?;
    Ok(())
}

pub fn run_with_stderr_to_stdout(
    mut command: std::process::Command,
) -> Result<std::process::ExitStatus> {
    command.stdout(Into::<std::process::Stdio>::into(os_pipe::dup_stderr()?));
    let mut spawned = command.spawn().unwrap();
    Ok(spawned.wait().unwrap())
}
