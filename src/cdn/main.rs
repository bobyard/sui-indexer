use rusoto_core::{Region, RusotoError};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() {
    let bucket_name = "your-bucket-name";
    let object_key = "your-object-key";
    let file_path = "./README.md";

    let mut file = File::open(file_path).unwrap();
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data).unwrap();

    let client = S3Client::new(Region::default());
    let request = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: object_key.to_owned(),
        body: Some(file_data.into()),
        ..Default::default()
    };

    match client.put_object(request).await {
        Ok(_) => println!("Image uploaded successfully"),
        Err(e) => println!("Error uploading image: {:?}", e),
    }
}

