use anyhow::Result;
use lapin::Connection;
use sui_indexer::indexer::receiver::create_exchange;
use tracing::error;

use crate::aws::S3Store;

use crate::PgPool;

use crate::token_worker::{
    batch_run_create_channel, handle_token_delete, handle_token_unwrap,
    handle_token_unwrap_when_delete, handle_token_update, handle_token_wrap,
};

pub struct Worker {
    s3: S3Store,
    pg: PgPool,
    mq: Connection,
}

impl Worker {
    pub fn new(s3: S3Store, pg: PgPool, mq: Connection) -> Self {
        Worker { s3, pg, mq }
    }

    pub async fn start(&mut self) -> Result<()> {
        let channel = self.mq.create_channel().await?;
        let _ = match create_exchange(channel).await {
            Ok(_) => tracing::info!("exchange created"),
            Err(e) => tracing::info!("error creating exchange: {}", e),
        };

        let update_channel = self.mq.create_channel().await?;
        let create_channel = self.mq.create_channel().await?;
        let delete_channel = self.mq.create_channel().await?;
        let wrap_channel = self.mq.create_channel().await?;
        let unwrap_channel = self.mq.create_channel().await?;
        let unwrap_when_delete_channel = self.mq.create_channel().await?;

        tokio::try_join!(
            handle_token_update(update_channel),
            batch_run_create_channel(
                20,
                create_channel,
                self.pg.clone(),
                self.s3.clone()
            ),
            handle_token_delete(delete_channel),
            handle_token_unwrap(unwrap_channel),
            handle_token_wrap(wrap_channel),
            handle_token_unwrap_when_delete(unwrap_when_delete_channel)
        )?;

        error!("all workers finished");
        Ok(())
    }
}
