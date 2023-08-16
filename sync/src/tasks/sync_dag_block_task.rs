use crate::{tasks::BlockFetcher, verified_rpc_client::VerifiedRpcClient};
use anyhow::{Ok, Result};
use futures::{future::BoxFuture, FutureExt};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_network_rpc_api::dag_protocol::{GetSyncDagBlockInfo, SyncDagBlockInfo};
use starcoin_storage::{
    block_info, flexi_dag::SyncFlexiDagSnapshotStorage, storage::CodecKVStore, Store,
};
use starcoin_types::block::Block;
use std::{collections::HashMap, sync::Arc};
use stream_task::{CollectorState, TaskResultCollector, TaskState};

use super::BlockLocalStore;

#[derive(Clone)]
pub struct SyncDagBlockTask {
    accumulator: Arc<MerkleAccumulator>,
    start_index: u64,
    batch_size: u64,
    target: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    local_store: Arc<dyn Store>,
}
impl SyncDagBlockTask {
    pub fn new(
        accumulator: MerkleAccumulator,
        start_index: u64,
        batch_size: u64,
        target: AccumulatorInfo,
        fetcher: Arc<VerifiedRpcClient>,
        accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
        local_store: Arc<dyn Store>,
    ) -> Self {
        SyncDagBlockTask {
            accumulator: Arc::new(accumulator),
            start_index,
            batch_size,
            target,
            fetcher,
            accumulator_snapshot: accumulator_snapshot.clone(),
            local_store: local_store.clone(),
        }
    }
}

impl SyncDagBlockTask {
    async fn fetch_absent_dag_block(&self, index: u64) -> Result<Vec<SyncDagBlockInfo>> {
        let leaf = self
            .accumulator
            .get_leaf(index)
            .expect(format!("index: {} must be valid", index).as_str())
            .expect(format!("index: {} should not be None", index).as_str());

        let snapshot = self
            .accumulator_snapshot
            .get(leaf)
            .expect(format!("index: {} must be valid for getting snapshot", index).as_str())
            .expect(format!("index: {} should not be None for getting snapshot", index).as_str());

        let block_with_infos = self
            .local_store
            .get_block_with_info(snapshot.child_hashes.clone())?;

        assert_eq!(block_with_infos.len(), snapshot.child_hashes.len());

        // the order must be the same between snapshot.child_hashes and block_with_infos
        let mut absent_block = vec![];
        let mut result = vec![];
        snapshot
            .child_hashes
            .iter()
            .zip(block_with_infos)
            .for_each(|(block_id, block_info)| {
                if let None = block_info {
                    absent_block.push(block_id.clone());
                    result.push(SyncDagBlockInfo {
                        block_id: block_id.clone(),
                        block: None,
                        absent_block: true,
                    })
                } else {
                    result.push(SyncDagBlockInfo {
                        block_id: block_id.clone(),
                        block: Some(block_info.unwrap().block),
                        absent_block: false,
                    })
                }
            });

        let fetched_block_info = self
            .fetcher
            .fetch_blocks(absent_block)
            .await?
            .iter()
            .map(|(block, peer_info)| (block.header().id(), (block.clone(), peer_info.clone())))
            .collect::<HashMap<_, _>>();

        // should return the block in order
        result.iter_mut().for_each(|block_info| {
            if block_info.absent_block {
                block_info.block = Some(fetched_block_info.get(&block_info.block_id).expect("the block should be got from peer already").0.to_owned());
            }
        });
        result.sort_by_key(|item| item.block_id);
        Ok(result)
    }
}

impl TaskState for SyncDagBlockTask {
    type Item = SyncDagBlockInfo;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            self.fetch_absent_dag_block(self.start_index).await
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_number = self.start_index.saturating_add(self.batch_size);
        if next_number > self.target.num_leaves {
            return None;
        }
        Some(Self {
            accumulator: self.accumulator.clone(),
            start_index: next_number,
            batch_size: self.batch_size,
            target: self.target.clone(),
            fetcher: self.fetcher.clone(),
            accumulator_snapshot: self.accumulator_snapshot.clone(),
            local_store: self.local_store.clone(),
        })
    }
}

pub struct SyncDagBlockCollector {
    chain: BlockChain,
}

impl SyncDagBlockCollector {
    pub fn new(chain: BlockChain) -> Self {
        Self { chain }
    }
}

impl TaskResultCollector<SyncDagBlockInfo> for SyncDagBlockCollector {
    type Output = ();

    fn collect(&mut self, mut _item: SyncDagBlockInfo) -> anyhow::Result<CollectorState> {
        Ok(CollectorState::Enough)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(())
    }
}
