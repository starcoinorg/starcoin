use anyhow::{anyhow, Result};
use clap::Clap;
use elasticsearch::auth::Credentials;
use elasticsearch::http::transport::SingleNodeConnectionPool;
use elasticsearch::http::Url;
use elasticsearch::Elasticsearch;
use futures_retry::{FutureRetry, RetryPolicy};
use futures_util::TryFutureExt;
use jsonrpc_core_client::transports::http;
use starcoin_indexer::{BlockClient, BlockData, EsSinker, IndexConfig};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::chain::ChainClient;
use std::time::Duration;
use tokio::runtime;

#[derive(Clap, Debug, Clone)]
#[clap(version = "0.1.0", author = "Starcoin Core Dev <dev@starcoin.org>")]
pub struct Options {
    #[clap(long, about = "es url", default_value = "http://localhost:9200")]
    es_url: Url,
    #[clap(long, about = "es user used to call api", requires = "es-password")]
    es_user: Option<String>,
    #[clap(long, about = "es user password")]
    es_password: Option<String>,
    #[clap(long, about = "es index prefix", default_value = "starcoin")]
    es_index_prefix: String,
    #[clap(
        long,
        about = "starcoin node rpc url",
        default_value = "http://localhost:9850"
    )]
    node_url: String,

    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Clap, Debug, Clone)]
enum SubCommand {
    Repair(Repair),
}

/// repair sub command
#[derive(Clap, Debug, Clone)]
struct Repair {
    // block to repair from. default to 0.
    #[clap(long = "from-block")]
    from_block: Option<u64>,

    // block to repair to. default to current end block
    #[clap(long = "to-block")]
    to_block: Option<u64>,
}

async fn start_loop(block_client: BlockClient, sinker: EsSinker) -> Result<()> {
    sinker.init_indices().await?;

    loop {
        let remote_tip_header = FutureRetry::new(
            || block_client.get_chain_head().map_err(|e| e),
            |e| {
                warn!("[Retry]: get chain head, err: {}", &e);
                RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
            },
        )
        .await
        .map(|(d, _)| d)
        .map_err(|(e, _)| e)?;

        let local_tip_header = FutureRetry::new(
            || sinker.get_local_tip_header(),
            |e| {
                warn!("[Retry]: get local tip header, err: {}", &e);
                RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
            },
        )
        .await
        .map(|(d, _)| d)
        .map_err(|(e, _)| e)?;

        let next_block_number = match local_tip_header.as_ref() {
            Some(local_tip_header) => local_tip_header.block_number + 1,
            None => 0,
        };
        if next_block_number > remote_tip_header.number.0 {
            tokio::time::delay_for(Duration::from_secs(1)).await;
        } else {
            let next_block: BlockData = FutureRetry::new(
                || {
                    block_client.get_block_whole_by_height(next_block_number)
                    //.map_err(|e| e.compat())
                },
                |e| {
                    warn!("[Retry]: get chain block data, err: {}", &e);
                    RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
                },
            )
            .await
            .map(|(d, _)| d)
            .map_err(|(e, _)| e)?;

            // fork occurs
            if let Some(local_tip_header) = local_tip_header.as_ref() {
                if next_block.block.header.parent_hash != local_tip_header.block_hash {
                    info!("Fork detected, rollbacking...");
                    FutureRetry::new(
                        || sinker.rollback_to_last_block(),
                        |e| {
                            warn!("[Retry]: rollback to last block, err: {}", &e);
                            RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
                        },
                    )
                    .await
                    .map(|(d, _)| d)
                    .map_err(|(e, _)| e)?;

                    continue;
                }
            }

            // retry write
            FutureRetry::new(
                || sinker.write_next_block(next_block.clone()),
                |e: anyhow::Error| {
                    warn!("[Retry]: write next block, err: {}", e);
                    RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
                },
            )
            .await
            .map(|(d, _)| d)
            .map_err(|(e, _)| e)?;

            info!(
                "Indexing block {}, height: {} done",
                next_block.block.header.block_hash, next_block.block.header.number
            );
        }
    }
}

async fn repair(block_client: BlockClient, sinker: EsSinker, repair_config: Repair) -> Result<()> {
    let latest_block_number = block_client
        .get_chain_head()
        .await
        .map_err(|e| anyhow!("{}", e))?
        .number
        .0;
    let end_block = repair_config.to_block.unwrap_or(latest_block_number);
    let from_block = repair_config.from_block.unwrap_or(0);
    let mut current_block: u64 = from_block;

    while current_block < end_block {
        let block_data: BlockData = FutureRetry::new(
            || {
                block_client.get_block_whole_by_height(current_block)
                //.map_err(|e| e.compat())
            },
            |e| {
                warn!("[Retry]: get chain block data, err: {}", &e);
                RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
            },
        )
        .await
        .map(|(d, _)| d)
        .map_err(|(e, _)| e)?;

        // retry write
        FutureRetry::new(
            || sinker.repair_block(block_data.clone()),
            |e: anyhow::Error| {
                warn!("[Retry]: repair block {}, err: {}", current_block, e);
                RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
            },
        )
        .await
        .map(|(d, _)| d)
        .map_err(|(e, _)| e)?;

        info!(
            "Repair block {}, height: {} done",
            block_data.block.header.block_hash, block_data.block.header.number
        );

        current_block += 1;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let _log_handle = starcoin_logger::init();
    let opts: Options = Options::parse();
    info!("opts: {:?}", &opts);
    let mut rt = runtime::Builder::new()
        .thread_name("starcoin-indexer")
        .threaded_scheduler()
        .enable_all()
        .build()?;
    let channel: ChainClient = rt
        .block_on(http::connect(opts.node_url.as_str()))
        .map_err(|e| anyhow!(format!("{}", e)))?;
    let block_client = BlockClient::new(channel);
    let mut transport = elasticsearch::http::transport::TransportBuilder::new(
        SingleNodeConnectionPool::new(opts.es_url),
    );
    if let Some(u) = opts.es_user.as_ref() {
        let user = u.clone();
        let pass = opts.es_password.unwrap_or_default();
        transport = transport.auth(Credentials::Basic(user, pass));
    }

    let transport = transport.build()?;
    let es = Elasticsearch::new(transport);
    let index_config = IndexConfig::new_with_prefix(opts.es_index_prefix.as_str());
    let sinker = EsSinker::new(es, index_config);

    match &opts.subcmd {
        Some(SubCommand::Repair(repair_config)) => {
            rt.block_on(repair(block_client, sinker, repair_config.clone()))?;
        }
        None => {
            rt.block_on(start_loop(block_client, sinker))?;
        }
    }
    Ok(())
}
