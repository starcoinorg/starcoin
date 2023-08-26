use crate::{tasks::BlockFetcher, verified_rpc_client::VerifiedRpcClient};
use anyhow::{Ok, Result};
use futures::{future::BoxFuture, FutureExt};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainWriter, ExecutedBlock};
use starcoin_network_rpc_api::dag_protocol::{GetSyncDagBlockInfo, SyncDagBlockInfo};
use starcoin_storage::{
    block_info, flexi_dag::SyncFlexiDagSnapshotStorage, storage::CodecKVStore, Store,
};
use starcoin_types::block::Block;
use std::{collections::HashMap, sync::Arc};
use stream_task::{CollectorState, TaskResultCollector, TaskState};

use super::{block_sync_task::SyncBlockData, BlockLocalStore};

#[derive(Clone)]
pub struct SyncDagBlockTask {
    accumulator: Arc<MerkleAccumulator>,
    start_index: u64,
    target: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    local_store: Arc<dyn Store>,
}
impl SyncDagBlockTask {
    pub fn new(
        accumulator: MerkleAccumulator,
        start_index: u64,
        target: AccumulatorInfo,
        fetcher: Arc<VerifiedRpcClient>,
        accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
        local_store: Arc<dyn Store>,
    ) -> Self {
        SyncDagBlockTask {
            accumulator: Arc::new(accumulator),
            start_index,
            target,
            fetcher,
            accumulator_snapshot: accumulator_snapshot.clone(),
            local_store: local_store.clone(),
        }
    }
}

impl SyncDagBlockTask {
    async fn fetch_absent_dag_block(&self, index: u64) -> Result<Vec<SyncBlockData>> {
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
                        peer_id: None,
                        dag_parents: vec![],
                        dag_transaction_header: None,
                    });
                } else {
                    result.push(SyncDagBlockInfo {
                        block_id: block_id.clone(),
                        block: Some(block_info.unwrap().block),
                        absent_block: false,
                        peer_id: None,
                        dag_parents: vec![],
                        dag_transaction_header: None,
                    });
                }
            });

        let fetched_block_info = self
            .fetcher
            .fetch_blocks(absent_block)
            .await?
            .iter()
            .map(|(block, peer_info, parents, transaction_header)| {
                (
                    block.header().id(),
                    (block.clone(), peer_info.clone(), parents.clone(), transaction_header.clone()),
                )
            })
            .collect::<HashMap<_, _>>();

        // should return the block in order
        result.iter_mut().for_each(|block_info| {
            if block_info.absent_block {
                block_info.block = Some(
                    fetched_block_info
                        .get(&block_info.block_id)
                        .expect("the block should be got from peer already")
                        .0
                        .to_owned(),
                );
                block_info.peer_id = fetched_block_info
                    .get(&block_info.block_id)
                    .expect("the block should be got from peer already")
                    .1
                    .to_owned();
                block_info.dag_parents = fetched_block_info
                    .get(&block_info.block_id)
                    .expect("the block should be got from peer already")
                    .2
                    .to_owned()
                    .expect("dag block should have parents");
                block_info.dag_transaction_header = Some(
                    fetched_block_info
                        .get(&block_info.block_id)
                        .expect("the block should be got from peer already")
                        .3
                        .to_owned()
                        .expect("dag block should have parents"));
            }
        });
        result.sort_by_key(|item| item.block_id);

        Ok(result
            .into_iter()
            .map(|item| {
                if !item.absent_block {
                    SyncBlockData {
                        block: item.block.expect("block should exists"),
                        info: Some(
                            self.local_store
                                .get_block_info(item.block_id)
                                .expect("failed to read block info")
                                .expect("block_info should exists"),
                        ),
                        peer_id: None,
                        accumulator_root: Some(
                            snapshot.accumulator_info.get_accumulator_root().clone(),
                        ),
                        count_in_leaf: snapshot.child_hashes.len() as u64,
                        dag_block_headers: Some(item.dag_parents),
                        dag_transaction_header: Some(
                            item.dag_transaction_header
                                .expect("dag transaction header should exists"),
                        ),
                    }
                } else {
                    SyncBlockData {
                        block: item.block.expect("block should exists"),
                        info: None,
                        peer_id: item.peer_id,
                        accumulator_root: Some(
                            snapshot.accumulator_info.get_accumulator_root().clone(),
                        ),
                        count_in_leaf: snapshot.child_hashes.len() as u64,
                        dag_block_headers: Some(item.dag_parents),
                        dag_transaction_header: Some(
                            item.dag_transaction_header
                                .expect("dag transaction header should exists"),
                        ),
                    }
                }
            })
            .collect())
    }
}

impl TaskState for SyncDagBlockTask {
    type Item = SyncBlockData;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move { self.fetch_absent_dag_block(self.start_index).await }.boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_number = self.start_index.saturating_add(1);
        if next_number >= self.target.num_leaves {
            return None;
        }
        Some(Self {
            accumulator: self.accumulator.clone(),
            start_index: next_number,
            target: self.target.clone(),
            fetcher: self.fetcher.clone(),
            accumulator_snapshot: self.accumulator_snapshot.clone(),
            local_store: self.local_store.clone(),
        })
    }
}
