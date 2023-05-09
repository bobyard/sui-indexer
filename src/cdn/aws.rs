use anyhow::{anyhow, Result};

use lazy_static::lazy_static;
use rusoto_core::credential::StaticProvider;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};

const REGION: Region = Region::UsWest1;
const BUCKET: &str = "bobyard";

lazy_static! {
    static ref KEY: String =
        std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set");
}

lazy_static! {
    static ref SECTRYKEY: String =
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set");
}

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
