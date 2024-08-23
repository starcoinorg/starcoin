use std::{collections::HashSet, result, sync::Arc};

use anyhow::Context;
use futures::FutureExt;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_chain::{verifier::DagVerifier, BlockChain};
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec};
use starcoin_logger::prelude::{error, info};
use starcoin_network_rpc_api::MAX_BLOCK_REQUEST_SIZE;
use starcoin_storage::{storage, Store};
use starcoin_types::block::{AccumulatorInfo, Block, BlockHeader};

use stream_task::{TaskResultCollector, TaskState};

use crate::store::{sync_absent_ancestor::DagSyncBlock, sync_dag_store::SyncDagStore};

use super::BlockFetcher;

#[derive(Clone)]
pub struct PrepareTheBlueBlockHash {
    sync_dag_store: SyncDagStore,
    read_size: usize,
}

impl PrepareTheBlueBlockHash {
    pub fn new(sync_dag_store: SyncDagStore, read_size: usize) -> Self {
        Self {
            sync_dag_store,
            read_size,
        }
    }
}

impl TaskState for PrepareTheBlueBlockHash {
    type Item = Block;

    fn new_sub_task(self) -> futures::future::BoxFuture<'static, anyhow::Result<Vec<Self::Item>>> {
        async move {
            let mut iter = self.sync_dag_store.iter_at_first()?;
            let mut sync_dag_blocks = vec![];
            self.sync_dag_store
                .read_by_iter(&mut iter, &mut sync_dag_blocks, self.read_size)?;
            anyhow::Ok(
                sync_dag_blocks
                    .into_iter()
                    .map(|sync_block| {
                        anyhow::Ok(
                            sync_block
                                .block
                                .ok_or_else(|| anyhow::format_err!("sync block is none"))?,
                        )
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?,
            )
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        Some({
            Self {
                sync_dag_store: self.sync_dag_store.clone(),
                read_size: self.read_size,
            }
        })
    }
}

pub struct ExecuteDagBlock {
    storage: Arc<dyn Store>,
    fetcher: Arc<dyn BlockFetcher>,
    sync_dag_store: SyncDagStore,
    time_service: Arc<dyn TimeService>,
    dag: BlockDAG,
}

impl ExecuteDagBlock {
    pub fn new(storage: Arc<dyn Store>, fetcher: Arc<dyn BlockFetcher>, sync_dag_store: SyncDagStore, time_service: Arc<dyn TimeService>, dag: BlockDAG) -> Self {
        Self {
            storage,
            fetcher,
            sync_dag_store,
            time_service,
            dag,
        }
    }
    async fn fetch_and_execute_absent_blocks(
        &mut self,
        block: Block,
        chain: &mut BlockChain,
    ) -> anyhow::Result<()> {
        // fetch the absent blocks
        let mut count = self.fetch_blocks(block.header().parents_hash()).await?;

        // go through the blocks and execute one by one until all are executed
        let sync_dag_store = self.sync_dag_store.clone();
        while count > 0 {
            let mut iter = sync_dag_store.iter_at_first()?;
            for result in iter.by_ref() {
                let (_, value) = result?;
                let sync_dag_block = DagSyncBlock::decode_value(&value)?;
                self.execute_block(sync_dag_block, chain)?;
                count = count.saturating_sub(1);
            }
        }
        anyhow::Ok(())
    }

    async fn fetch_blocks(&self, mut parents: Vec<HashValue>) -> anyhow::Result<usize> {
        let mut count = 0usize;
        loop {
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
                break;
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

            count = count.saturating_add(blocks.len());

            blocks
                .iter()
                .try_for_each(|block| self.sync_dag_store.save_block(block.clone()))?;

            parents = blocks
                .into_iter()
                .flat_map(|block| block.header().parents_hash())
                .collect();
        }
        anyhow::Ok(count)
    }

    fn execute_block(
        &mut self,
        sync_blocks: DagSyncBlock,
        chain: &mut BlockChain,
    ) -> anyhow::Result<()> {
        let block = sync_blocks
            .block
            .ok_or_else(|| anyhow::format_err!("failed to unwrap the sync dag block"))?;

        if block.header().parents_hash().into_iter().any(|parent_id| {
            match self.storage.get_block_info(parent_id) {
                Ok(op_block_info) => {
                    if op_block_info.is_none() {
                        info!(
                            "block: {:?} 's parent_id: {:?} is not executed, waiting",
                            block.header(),
                            parent_id
                        );
                        true
                    } else {
                        false
                    }
                }
                Err(e) => {
                    error!(
                        "failed to get the block info by id: {:?}, error: {:?}",
                        parent_id, e
                    );
                    true
                }
            }
        }) {
            info!(
                "waiting the parent block executed, header: {:?}",
                block.header()
            );
            return anyhow::Ok(());
        }

        let verified_block = chain.verify_with_verifier::<DagVerifier>(block)?;

        let executed_block = chain.execute_block_without_dag_commit(verified_block)?;

        self.sync_dag_store.delete_dag_sync_block(
            executed_block.block.header().number(),
            executed_block.block.id(),
        )?;

        anyhow::Ok(())
    }
}

impl TaskResultCollector<Block> for ExecuteDagBlock {
    type Output = ();

    fn collect(&mut self, block: Block) -> anyhow::Result<stream_task::CollectorState> {
        let mut chain = BlockChain::new(
            self.time_service.clone(),
            block.id(),
            self.storage.clone(),
            None,
            self.dag.clone(),
        )?;
        async_std::task::block_on(self.fetch_and_execute_absent_blocks(block, &mut chain))?;
        anyhow::Ok(stream_task::CollectorState::Need)
    }

    fn finish(self) -> anyhow::Result<Self::Output> {
        Ok(())
    }
}
