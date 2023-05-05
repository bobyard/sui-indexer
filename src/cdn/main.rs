mod aws;

use anyhow::Result;
use bytes::Buf;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut s3 = aws::S3Store::new();

    let bytes = read_to_buffer(
        "https://gateway.pinata.cloud/ipfs/QmXiSJPXJ8mf9LHijv6xFH1AtGef4h8v5VPEKZgjR4nzvM",
    )
    .await?;

    s3.upload_images_to_s3("test".to_string(), bytes).await?;

    Ok(())
}

async fn read_to_buffer(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let mut buffer = Vec::new();
    let mut reader = bytes.reader();
    let _ = std::io::copy(&mut reader, &mut buffer)?;

    Ok(buffer)
}
