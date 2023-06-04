use anyhow::{anyhow, Result};

use bytes::Buf;
use lazy_static::lazy_static;
use rusoto_core::credential::StaticProvider;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};

const REGION: Region = Region::UsWest1;
const BUCKET: &str = "bobyard";
const IPFS_GATEWAY: &str = "https://gateway.ipfs.io/ipfs/";

lazy_static! {
    static ref KEY: String = std::env::var("AWS_ACCESS_KEY_ID")
        .expect("AWS_ACCESS_KEY_ID must be set");
}

lazy_static! {
    static ref SECTRYKEY: String = std::env::var("AWS_SECRET_ACCESS_KEY")
        .expect("AWS_SECRET_ACCESS_KEY must be set");
}

#[derive(Clone)]
pub struct S3Store {
    bucket_name: String,
    client: S3Client,
}

impl S3Store {
    pub fn new() -> Self {
        let credentials = rusoto_core::credential::AwsCredentials::new(
            KEY.to_string(),
            SECTRYKEY.to_string(),
            None,
            None,
        );

        let credentials_provider = StaticProvider::from(credentials);
        let client = S3Client::new_with(
            rusoto_core::request::HttpClient::new().unwrap(),
            credentials_provider,
            REGION,
        );
        S3Store {
            client,
            bucket_name: BUCKET.to_string(),
        }
    }

    pub async fn find_exist_in_s3(&mut self, object_key: String) -> Result<()> {
        let request = GetObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: object_key.to_owned(),
            ..Default::default()
        };

        let _ = self.client.get_object(request).await?;
        Ok(())
    }

    pub async fn read_to_buffer(
        &mut self,
        url: &str,
    ) -> Result<(Vec<u8>, String)> {
        let response = reqwest::get(url).await?.error_for_status()?;
        if !response.status().is_success() && response.status().is_redirection()
        {
            return Err(anyhow!("too many requests"));
        }

        if !response.status().is_success() {
            return Err(anyhow!("download failed {}", response.status()));
        }

        let mut format = "".to_string();

        let headers = response.headers();
        if headers.contains_key("content-type") {
            format = headers.get("content-type").unwrap().to_str()?.to_string();
        }

        let bytes = response.bytes().await?;
        let mut buffer = Vec::new();
        let mut reader = bytes.reader();
        let _ = std::io::copy(&mut reader, &mut buffer)?;

        Ok((buffer, format))
    }

    pub async fn update_with_remote_url(
        &mut self,
        mut url: String,
    ) -> Result<String> {
        let mut img = "".to_string();

        if url.starts_with("ipfs://") {
            url =
                IPFS_GATEWAY.to_string() + url.strip_prefix("ipfs://").unwrap();
        }

        if url != "" {
            let (buffer, format) = self.read_to_buffer(&url).await?;
            tracing::info!(
                "download form url {} respformat: {}",
                &url,
                &format
            );

            let name = blake3::hash(&buffer);
            tracing::info!("name:{}", name.to_string());

            let res = self.find_exist_in_s3(name.to_string()).await;
            if res.is_err() {
                let mine = if format.is_empty() {
                    if url.ends_with("svg") {
                        Some("image/svg+xml".to_string())
                    } else if url.ends_with("png") {
                        Some("image/png".to_string())
                    } else if url.ends_with("jpg") || url.ends_with("jpeg") {
                        Some("image/jpeg".to_string())
                    } else if url.ends_with("gif") {
                        Some("image/gif".to_string())
                    } else if url.ends_with("webp") {
                        Some("image/webp".to_string())
                    } else {
                        None
                    }
                } else {
                    Some(format)
                };

                //not exist in s3, we upload it
                let res = self
                    .upload_images_to_s3(name.to_string(), buffer, mine)
                    .await;
                if res.is_ok() {
                    img = name.to_string();
                } else {
                    tracing::info!(
                        "upload image to s3 error: {:?} url: {:?}",
                        res,
                        url
                    );
                };
            } else {
                img = name.to_string();
            }
        }

        Ok(img)
    }

    pub async fn upload_images_to_s3(
        &mut self,
        object_key: String,
        file_data: Vec<u8>,
        mut mime: Option<String>,
    ) -> Result<()> {
        if mime.is_none() {
            let img = image::guess_format(&file_data)?;
            let ext = img
                .extensions_str()
                .get(0)
                .ok_or(anyhow!("Can't get extensions for the file"))?;
            let guest_mine = mime_guess::from_ext(ext)
                .first()
                .ok_or(anyhow!("Can't guess the mine for ext"))?
                .to_string();

            mime = Some(guest_mine);
        }

        let request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: object_key.to_owned(),
            body: Some(file_data.into()),
            content_type: mime,
            ..Default::default()
        };

        let res = match self.client.put_object(request).await {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Error uploading file: {:?}", e)),
        };

        Ok(res)
    }
}
