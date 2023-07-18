use anyhow::{Result, anyhow};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

#[tokio::main]
pub async fn download_to(dest_path: &std::path::PathBuf, url: &str) -> Result<()> {
    let tmp_dest_path = dest_path.clone().with_extension("tmp");
    let mut dest_file = std::fs::File::create(&tmp_dest_path)?;

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let response = client.get(url).send().await?;
    if let Err(err) = &response.error_for_status_ref() {
        return Err(anyhow!("Failed to download file: {}", err));
    }

    let mut cursor = std::io::Cursor::new(response.bytes().await?);
    std::io::copy(&mut cursor, &mut dest_file)?;
    std::fs::rename(tmp_dest_path, dest_path)?;
    Ok(())
}
