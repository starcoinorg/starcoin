use anyhow::{bail, Result};
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_service_registry::ServiceRef;

use crate::head_block;

#[derive(Debug, Clone)]
pub struct BlockWindowStats {
    pub block_count: u64,
    pub txn_count: usize,
    pub duration_secs: f64,
    pub tps: f64,
}

#[derive(Debug, Clone)]
pub struct PostPrepareBlockStats {
    pub start_block: u64,
    pub end_block: u64,
    pub txn_count: usize,
}

pub async fn recent_block_window_stats(
    chain_reader: ServiceRef<ChainReaderService>,
    window: u64,
) -> Result<Option<BlockWindowStats>> {
    if window == 0 {
        return Ok(None);
    }

    let head = head_block(chain_reader.clone()).await?;
    let head_number = head.header.number();
    // Skip genesis (block 0) to avoid huge time spans on fresh chains.
    if head_number <= 1 {
        return Ok(None);
    }

    // never include block 0 (genesis) in the window
    let start_number = std::cmp::max(1, head_number.saturating_sub(window.saturating_sub(1)));
    let mut first_ts: Option<u64> = None;
    let mut last_ts: Option<u64> = None;
    let mut txn_count: usize = 0;
    let mut block_count: u64 = 0;

    for number in start_number..=head_number {
        if let Some(block) = block_by_number(number, chain_reader.clone()).await? {
            block_count += 1;
            let ts = block.header.timestamp();
            if first_ts.is_none() {
                first_ts = Some(ts);
            }
            last_ts = Some(ts);
            txn_count += block.body.transactions.len();
            txn_count += block.body.transactions2.len();
        }
    }

    if block_count == 0 {
        return Ok(None);
    }

    let duration_secs = match (first_ts, last_ts) {
        (Some(start), Some(end)) if end > start => (end - start) as f64 / 1000.0,
        _ => 0.0,
    };
    let tps = if duration_secs > 0.0 {
        txn_count as f64 / duration_secs
    } else {
        txn_count as f64
    };

    Ok(Some(BlockWindowStats {
        block_count,
        txn_count,
        duration_secs,
        tps,
    }))
}

pub async fn user_tx_since_block(
    start_exclusive: u64,
    chain_reader: ServiceRef<ChainReaderService>,
) -> Result<PostPrepareBlockStats> {
    let head = head_block(chain_reader.clone()).await?;
    let head_number = head.header.number();
    let txn_count = count_user_tx_in_range(start_exclusive, head_number, chain_reader).await?;
    Ok(PostPrepareBlockStats {
        start_block: start_exclusive,
        end_block: head_number,
        txn_count,
    })
}

pub(crate) async fn count_user_tx_in_range(
    start_exclusive: u64,
    end_inclusive: u64,
    chain_reader: ServiceRef<ChainReaderService>,
) -> Result<usize> {
    if end_inclusive <= start_exclusive {
        return Ok(0);
    }
    let mut total = 0usize;
    for number in (start_exclusive + 1)..=end_inclusive {
        if let Some(block) = block_by_number(number, chain_reader.clone()).await? {
            total += block.body.transactions.len();
            total += block.body.transactions2.len();
        }
    }
    Ok(total)
}

pub async fn block_by_number(
    number: u64,
    chain_reader: ServiceRef<ChainReaderService>,
) -> Result<Option<starcoin_types::block::Block>> {
    match chain_reader
        .send(ChainRequest::GetBlockByNumber(number))
        .await??
    {
        ChainResponse::BlockOption(b) => Ok(b.map(|b| *b)),
        _ => bail!("unexpected block-by-number response"),
    }
}
