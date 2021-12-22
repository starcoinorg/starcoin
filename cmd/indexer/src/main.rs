use anyhow::{anyhow, Result};
use clap::Parser;
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
use std::cmp::min;
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime;

#[derive(Parser, Debug, Clone)]
#[clap(version = "0.1.0", author = "Starcoin Core Dev <dev@starcoin.org>")]
pub struct Options {
    #[clap(long, help = "es url", default_value = "http://localhost:9200")]
    es_url: Url,
    #[clap(long, help = "es user used to call api", requires = "es-password")]
    es_user: Option<String>,
    #[clap(long, help = "es user password")]
    es_password: Option<String>,
    #[clap(long, help = "es index prefix", default_value = "starcoin")]
    es_index_prefix: String,
    #[clap(
        long,
        help = "starcoin node rpc url",
        default_value = "http://localhost:9850"
    )]
    node_url: String,
    #[clap(long, help = "es bulk size", default_value = "50")]
    bulk_size: u64,

    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Repair(Repair),
}

/// repair sub command
#[derive(Parser, Debug, Clone)]
struct Repair {
    // block to repair from. default to 0.
    #[clap(long = "from-block")]
    from_block: Option<u64>,

    // block to repair to. default to current end block
    #[clap(long = "to-block")]
    to_block: Option<u64>,
}

async fn start_loop(block_client: BlockClient, sinker: EsSinker, bulk_size: u64) -> Result<()> {
    sinker.init_indices().await?;

    loop {
        let chain_header = FutureRetry::new(
            || block_client.get_chain_head().map_err(|e| e),
            |e| {
                warn!("[Retry]: get chain head, err: {}", &e);
                RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
            },
        )
        .await
        .map(|(d, _)| d)
        .map_err(|(e, _)| e)?;

        let local_tip_header = sinker.get_local_tip_header().await?;
        let current_block_number = match local_tip_header.as_ref() {
            Some(local_tip_header) => {
                //sleep
                if chain_header.number.0 == local_tip_header.block_number {
                    sleep(Duration::from_secs(1));
                }
                local_tip_header.block_number
            }
            None => 0,
        };
        info!("current_block_number: {}", current_block_number);
        let bulk_times = min(chain_header.number.0 - current_block_number, bulk_size);
        let mut block_vec = vec![];
        let mut index = 1u64;

        while index <= bulk_times {
            let read_number = current_block_number + index;
            let next_block: BlockData = FutureRetry::new(
                || {
                    block_client.get_block_whole_by_height(read_number)
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
            let local_tip_header = sinker.get_local_tip_header().await?;

            if let Some(local_tip_header) = local_tip_header.as_ref() {
                if next_block.block.header.parent_hash != local_tip_header.block_hash
                    && read_number > 0
                {
                    // fork occurs
                    warn!(
                        "Fork detected, rollbacking: {}, {}, {}",
                        read_number,
                        next_block.block.header.parent_hash,
                        local_tip_header.block_hash
                    );
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
                    break;
                }
            }
            block_vec.push(next_block.clone());
            sinker
                .update_local_tip_header(
                    next_block.block.header.block_hash,
                    next_block.block.header.number.0,
                )
                .await?;
            index += 1;
            info!(
                "Indexing block {}, height: {}",
                next_block.block.header.block_hash, next_block.block.header.number
            );
        }

        if index >= bulk_times {
            // bulk send
            FutureRetry::new(
                || sinker.bulk(block_vec.clone()),
                |e: anyhow::Error| {
                    warn!("[Retry]: write next blocks, err: {}", e);
                    RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
                },
            )
            .await
            .map(|(d, _)| d)
            .map_err(|(e, _)| e)?;
            let local_tip_header = sinker.get_local_tip_header().await?;
            if let Some(tip_info) = local_tip_header.as_ref() {
                FutureRetry::new(
                    || sinker.update_remote_tip_header(tip_info.block_hash, tip_info.block_number),
                    |e: anyhow::Error| {
                        warn!("[Retry]: write next blocks, err: {}", e);
                        RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
                    },
                )
                .await
                .map(|(d, _)| d)
                .map_err(|(e, _)| e)?;
            }
            info!("Indexing height: {} done", current_block_number + index);
        }
        block_vec.clear();
    }
}

async fn repair(
    block_client: BlockClient,
    sinker: EsSinker,
    repair_config: Repair,
    bulk_size: u64,
) -> Result<()> {
    let latest_block_number = block_client
        .get_chain_head()
        .await
        .map_err(|e| anyhow!("{}", e))?
        .number
        .0;
    let end_block = repair_config.to_block.unwrap_or(latest_block_number);
    let from_block = repair_config.from_block.unwrap_or(0);
    let mut current_block: u64 = from_block;
    let mut block_vec = vec![];
    let mut index = 0;
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

        block_vec.push(block_data.clone());
        debug!(
            "Repair block {}, height: {} commit",
            block_data.block.header.block_hash, block_data.block.header.number
        );
        index += 1;
        if index >= bulk_size {
            // retry write
            FutureRetry::new(
                || sinker.bulk(block_vec.clone()),
                |e: anyhow::Error| {
                    warn!("[Retry]: repair block {}, err: {}", current_block, e);
                    RetryPolicy::<anyhow::Error>::WaitRetry(Duration::from_secs(1))
                },
            )
            .await
            .map(|(d, _)| d)
            .map_err(|(e, _)| e)?;
            info!(
                "repair block {}, {} done.",
                block_data.block.header.block_hash, block_data.block.header.number
            );
            block_vec.clear();
            index = 0;
        }
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
    let bulk_size = opts.bulk_size;

    match &opts.subcmd {
        Some(SubCommand::Repair(repair_config)) => {
            rt.block_on(repair(
                block_client,
                sinker,
                repair_config.clone(),
                bulk_size,
            ))?;
        }
        None => {
            rt.block_on(start_loop(block_client, sinker, bulk_size))?;
        }
    }
    Ok(())
}
