use anyhow::Result;

fn is_request_retryable_based_on_error(err: &reqwest::Error) -> bool {
    if err.is_timeout() || err.is_connect() {
        return true;
    }

    if err.is_decode() || err.is_body() || err.is_builder() || err.is_request() || err.is_redirect()
    {
        return false;
    }

    true
}

pub fn download_to(dest_path: &std::path::PathBuf, url: &str) -> Result<()> {
    let tmp_dest_path = dest_path.clone().with_extension("tmp");
    let mut dest_file = std::fs::File::create(&tmp_dest_path)?;

    let mut num_tries = 5;
    let mut wait_duration = std::time::Duration::from_millis(200);
    let mut response = reqwest::blocking::get(url);
    while let Err(err) = &response {
        if num_tries < 0 || !is_request_retryable_based_on_error(err) {
            break;
        }

        std::thread::sleep(wait_duration);
        wait_duration *= 2;
        num_tries -= 1;

        response = reqwest::blocking::get(url);
    }

    let response = response?;
    let mut content = std::io::Cursor::new(response.bytes()?);
    std::io::copy(&mut content, &mut dest_file)?;
    std::fs::rename(tmp_dest_path, dest_path)?;
    Ok(())
}
