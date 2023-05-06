use crate::aws::S3Store;
use anyhow::Result;
use bytes::Buf;
use diesel::connection::DefaultLoadingMode;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use sui_indexer::models::tokens as model_tokens;
use sui_indexer::schema::tokens::dsl::*;
use tracing::error;

use sui_indexer::models;
use sui_indexer::models::tokens::{query_the_uncache_images, update_image_url};

const IPFS_GATEWAY: &str = "https://cloudflare-ipfs.com/ipfs/";

pub async fn run(s3: &mut S3Store, pg: &mut PgConnection) -> Result<()> {
    let uncache = query_the_uncache_images(pg)?;

    for item in &uncache {
        let mut url = item.metadata_uri.clone();
        let mut img: Option<String> = None;

        if url.starts_with("ipfs://") {
            url = IPFS_GATEWAY.to_string() + url.strip_prefix("ipfs://").unwrap();
        }

        if url != "" {
            let buffer = match read_to_buffer(&url).await {
                Ok(buffer) => buffer,
                Err(e) => {
                    //TODO fix all this
                    error!("read to buffer error: {:?} url: {:?}", e, url);
                    continue;
                }
            };

            let name = blake3::hash(&buffer);
            let res = s3.find_exist_in_s3(name.to_string()).await;
            if res.is_err() {
                let mine = if url.ends_with(".svg") {
                    Some("image/svg+xml".to_string())
                } else {
                    None
                };

                //not exist in s3, we upload it
                let res = s3.upload_images_to_s3(name.to_string(), buffer, mine).await;
                if res.is_ok() {
                    img = Some(name.to_string());
                } else {
                    error!("upload image to s3 error: {:?} url: {:?}", res, url);
                };
            } else {
                img = Some(name.to_string());
            }
        }

        // the sourece has a problem. so We take this to avoid the error
        if img.is_none() {
            img = Some("".to_string());
        }

        update_image_url(pg, item.token_id.clone(), img)?;
    }

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
