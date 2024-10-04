use anyhow::{anyhow, Result};

fn is_request_retryable_based_on_error(err: &reqwest::Error) -> bool {
    if err.is_timeout() || err.is_connect() {
        return true;
    }

    if err.is_decode() || err.is_body() || err.is_builder() || err.is_request() || err.is_redirect()
    {
        return false;
    }

    if let Some(code) = err.status() {
        return match code.as_u16() {
            408 | 425 | 429 | 500 | 502 | 503 | 504 => true, // https://stackoverflow.com/a/74627395/21956251
            _ => false,
        };
    }

    true
}

pub fn download_to(dest_path: &std::path::PathBuf, url: &str) -> Result<()> {
    let tmp_dest_path = dest_path.clone().with_extension("tmp");
    let mut dest_file = std::fs::File::create(&tmp_dest_path)?;

    // The final attempt will wait 12 seconds before proceeding
    let num_tries = 8;
    let wait_duration = std::time::Duration::from_millis(200);

    for attempt_num in 0..num_tries {
        if attempt_num > 0 {
            let this_wait_duration = wait_duration * 2_u32.pow(attempt_num - 1);
            eprintln!(
                "Retrying, attempt #{} after sleeping for {}ms",
                attempt_num,
                this_wait_duration.as_millis()
            );
            std::thread::sleep(this_wait_duration);
        }

        let response = reqwest::blocking::get(url);
        if let Err(err) = &response {
            eprintln!("Error while making GET request: {}", err);
            if is_request_retryable_based_on_error(err) {
                continue;
            }
            break;
        }

        let response = response?;
        if let Err(err) = &response.error_for_status_ref() {
            eprintln!("GET request failed: {}", err);
            if is_request_retryable_based_on_error(err) {
                continue;
            }
            break;
        }

        let content = response.bytes();
        if let Err(err) = &content {
            eprintln!("Error while making streaming response: {}", err);
            if is_request_retryable_based_on_error(err) {
                continue;
            }
            break;
        }

        let mut cursor = std::io::Cursor::new(content?);
        std::io::copy(&mut cursor, &mut dest_file)?;
        std::fs::rename(tmp_dest_path, dest_path)?;
        return Ok(());
    }

    Err(anyhow!("Failed to download {}", url))
}
