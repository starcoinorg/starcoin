use std::{collections::HashSet, result, sync::Arc};

use anyhow::Context;
use futures::FutureExt;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_chain::{verifier::DagVerifier, BlockChain};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::error;
use starcoin_network_rpc_api::MAX_BLOCK_REQUEST_SIZE;
use starcoin_storage::Store;
use starcoin_types::block::{AccumulatorInfo, Block, BlockHeader};

use stream_task::{TaskResultCollector, TaskState};

use crate::store::sync_dag_store::SyncDagStore;

use super::BlockFetcher;

#[derive(Clone)]
struct PrepareTheBlueBlockHash {
    storage: Arc<dyn Store>,
    block_accumulator_info: AccumulatorInfo,
    start_number: u64,
    step_size: u64,
}

impl TaskState for PrepareTheBlueBlockHash {
    type Item = HashValue;

    fn new_sub_task(self) -> futures::future::BoxFuture<'static, anyhow::Result<Vec<Self::Item>>> {
        async move {
            let block_accumulator = MerkleAccumulator::new_with_info(
                self.block_accumulator_info,
                self.storage
                    .get_accumulator_store(AccumulatorStoreType::Block),
            );
            block_accumulator.get_leaves(self.start_number, false, self.step_size)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        Some(Self {
            storage: self.storage.clone(),
            block_accumulator_info: self.block_accumulator_info.clone(),
            start_number: self.start_number.saturating_add(self.step_size),
            step_size: self.step_size,
        })
    }
}

struct ExecuteDagBlock {
    storage: Arc<dyn Store>,
    fetcher: Arc<dyn BlockFetcher>,
    headers: Vec<BlockHeader>,
    blocks: Vec<Block>,
    target_id: HashValue,
    batch_size: usize,
    sync_dag_store: SyncDagStore,
    chain: BlockChain,
}

impl ExecuteDagBlock {
    fn fetch_blocks(&mut self, mut parents: Vec<HashValue>) -> anyhow::Result<Vec<BlockHeader>> {
        async_std::task::block_on(async move {
            parents.retain(|header_id| match self.storage.get_block_info(*header_id) {
                Ok(op_block_info) => op_block_info.is_none(),
                Err(e) => {
                    error!(
                        "failed to get the block info by id: {:?}, error: {:?}",
                        header_id, e
                    );
                    true
                }
            });
            if parents.is_empty() {
                return Ok(vec![]);
            }
            let mut blocks = vec![];
            for request_ids in parents.chunks(usize::try_from(MAX_BLOCK_REQUEST_SIZE)?) {
                blocks.extend(
                    self.fetcher
                        .fetch_blocks(request_ids.to_vec())
                        .await?
                        .into_iter()
                        .map(|(block, _)| block),
                );
            }

            blocks
                .iter()
                .try_for_each(|block| self.sync_dag_store.save_block(block.clone()))?;

            anyhow::Ok(
                blocks
                    .into_iter()
                    .map(|block| block.header().clone())
                    .collect(),
            )
        })
    }

    fn fetch_and_save_absent_blocks(&mut self) -> anyhow::Result<()> {
        let headers = std::mem::take(&mut self.headers);
        let mut parents = headers
            .into_iter()
            .flat_map(|header| header.parents_hash())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        loop {
            let headers = self.fetch_blocks(parents.clone())?;
            if headers.is_empty() {
                break;
            }
            parents = std::mem::take(
                &mut headers
                    .into_iter()
                    .flat_map(|header| header.parents_hash())
                    .collect::<HashSet<_>>(),
            )
            .into_iter()
            .collect::<Vec<_>>();
        }

        anyhow::Ok(())
    }

    fn execute_block(&mut self) -> anyhow::Result<()> {
        let sync_dag_store = self.sync_dag_store.clone();
        let mut absent_block_iter = sync_dag_store
            .iter_at_first()
            .context("Failed to create iterator for sync_dag_store")?;
        let mut local_absent_block = vec![];
        loop {
            sync_dag_store.read_by_iter(&mut absent_block_iter, &mut local_absent_block, 720)?;
            local_absent_block.iter().try_for_each(|block| {
                let block = block
                    .block
                    .as_ref()
                    .ok_or_else(|| anyhow::format_err!("failed to unwrap the sync dag block"))?
                    .clone();
                self.sync_dag_store
                    .delete_dag_sync_block(block.header().number(), block.id())?;

                let verified_block = self.chain.verify_with_verifier::<DagVerifier>(block)?;
                self.chain
                    .execute_block_without_dag_commit(verified_block)?;

                anyhow::Ok(())
            })?;
            local_absent_block.retain(|dag_sync_block| {
                if dag_sync_block.block.is_none() {
                    return false;
                }
                match self.storage.get_block_info(
                    dag_sync_block
                        .block
                        .as_ref()
                        .expect("dag block is not none")
                        .header()
                        .id(),
                ) {
                    Ok(op_block_info) => op_block_info.is_none(),
                    Err(e) => {
                        error!("failed to unwrap the sync dag block by error: {:?}", e);
                        true
                    }
                }
            });
            if local_absent_block.is_empty() {
                break;
            }
        }
        Ok(())
    }
}

impl TaskResultCollector<HashValue> for ExecuteDagBlock {
    type Output = ();

    fn collect(&mut self, block_id: HashValue) -> anyhow::Result<stream_task::CollectorState> {
        self.headers.push(
            self.storage
                .get_block_header_by_hash(block_id)?
                .ok_or_else(|| anyhow::format_err!("block header not found by id: {}", block_id))?,
        );

        if block_id == self.target_id {
            self.fetch_and_save_absent_blocks()?;
            self.execute_block()?;
            anyhow::Ok(stream_task::CollectorState::Enough)
        } else {
            if self.batch_size <= self.headers.len() {
                self.fetch_and_save_absent_blocks()?;
            }
            anyhow::Ok(stream_task::CollectorState::Need)
        }
    }

    fn finish(self) -> anyhow::Result<Self::Output> {
        Ok(())
    }
}
