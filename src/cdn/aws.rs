use anyhow::{anyhow, Result};
use image::io::Reader as ImageReader;
use lazy_static::lazy_static;
use rusoto_core::credential::StaticProvider;
use rusoto_core::{Client, Region, RusotoError};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::fs::File;
use std::io::Cursor;
use std::io::Read;

use dotenv::dotenv;

const REGION: Region = Region::UsWest1;
const BUCKET: &str = "bobyard";

lazy_static! {
    static ref KEY: String =
        { std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set") };
    static ref SECTRYKEY: String =
        { std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set") };
}

pub(crate) struct S3Store {
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

    pub async fn upload_images_to_s3(
        &mut self,
        object_key: String,
        file_data: Vec<u8>,
    ) -> Result<()> {
        let img = image::guess_format(&file_data)?;
        let ext = img.extensions_str().get(0).unwrap();
        let mine = mime_guess::from_ext(ext).first().unwrap().to_string();

        let request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: object_key.to_owned(),
            body: Some(file_data.into()),
            content_type: Some(mine),
            ..Default::default()
        };

        let res = match self.client.put_object(request).await {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Error uploading file: {:?}", e)),
        };

        Ok(res)
    }
}
