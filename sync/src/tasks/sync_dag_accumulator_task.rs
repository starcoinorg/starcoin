use anyhow::{bail, ensure, Chain, Result};
use bcs_ext::BCSCodec;
use futures::{future::BoxFuture, FutureExt};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_network_rpc_api::dag_protocol::{self, TargetDagAccumulatorLeafDetail};
use starcoin_storage::{
    flexi_dag::{SyncFlexiDagSnapshot, SyncFlexiDagSnapshotStorage},
    storage::CodecKVStore,
};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

use crate::verified_rpc_client::VerifiedRpcClient;

#[derive(Clone)]
pub struct SyncDagAccumulatorTask {
    leaf_index: u64,
    batch_size: u64,
    target_index: u64,
    fetcher: Arc<VerifiedRpcClient>,
}
impl SyncDagAccumulatorTask {
    pub fn new(
        leaf_index: u64,
        batch_size: u64,
        target_index: u64,
        fetcher: Arc<VerifiedRpcClient>,
    ) -> Self {
        SyncDagAccumulatorTask {
            leaf_index,
            batch_size,
            target_index,
            fetcher,
        }
    }
}

impl TaskState for SyncDagAccumulatorTask {
    type Item = TargetDagAccumulatorLeafDetail;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let target_details = match self
                .fetcher
                .get_accumulator_leaf_detail(dag_protocol::GetTargetDagAccumulatorLeafDetail {
                    leaf_index: self.leaf_index,
                    batch_size: self.batch_size,
                })
                .await?
            {
                Some(details) => details,
                None => {
                    bail!("return None when sync accumulator for dag");
                }
            };
            Ok(target_details)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        //this should never happen, because all node's genesis block should same.
        if self.leaf_index == 0 {
            // it is genesis
            return None;
        }

        let next_number = self.leaf_index.saturating_add(self.batch_size);
        if next_number > self.target_index - 1 {
            // genesis leaf doesn't need synchronization
            return None;
        }
        Some(Self {
            fetcher: self.fetcher.clone(),
            leaf_index: next_number,
            batch_size: self.batch_size,
            target_index: self.target_index,
        })
    }
}

pub struct SyncDagAccumulatorCollector {
    accumulator: MerkleAccumulator,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    target: AccumulatorInfo,
    start_leaf_index: u64,
}

impl SyncDagAccumulatorCollector {
    pub fn new(
        accumulator: MerkleAccumulator,
        accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
        target: AccumulatorInfo,
        start_leaf_index: u64,
    ) -> Self {
        Self {
            accumulator,
            accumulator_snapshot,
            target,
            start_leaf_index,
        }
    }
}

impl TaskResultCollector<TargetDagAccumulatorLeafDetail> for SyncDagAccumulatorCollector {
    type Output = (u64, MerkleAccumulator);

    fn collect(
        &mut self,
        mut item: TargetDagAccumulatorLeafDetail,
    ) -> anyhow::Result<CollectorState> {
        let accumulator_leaf = BlockChain::calculate_dag_accumulator_key(item.tips.clone())?;
        self.accumulator.append(&[accumulator_leaf])?;
        let accumulator_info = self.accumulator.get_info();
        if accumulator_info.accumulator_root != item.accumulator_root {
            bail!(
                "sync occurs error for the accumulator root differs from other!, local {}, peer {}",
                accumulator_info.accumulator_root,
                item.accumulator_root
            )
        }
        self.accumulator.flush()?;

        let num_leaves = accumulator_info.num_leaves;
        self.accumulator_snapshot.put(
            accumulator_leaf,
            SyncFlexiDagSnapshot {
                child_hashes: item.tips.clone(),
                accumulator_info: accumulator_info.clone(),
            },
        )?;

        item.tips.iter().try_fold((), |_, block_id| {
            self.accumulator_snapshot.put(
                block_id.clone(),
                SyncFlexiDagSnapshot {
                    child_hashes: item.tips.clone(),
                    accumulator_info: accumulator_info.clone(),
                },
            )
        })?;

        if num_leaves == self.target.num_leaves {
            Ok(CollectorState::Enough)
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(self) -> Result<Self::Output> {
        let accumulator_info = self.accumulator.get_info();

        ensure!(
            accumulator_info == self.target,
            "local accumulator info: {:?}, peer's: {:?}",
            accumulator_info,
            self.target
        );
        info!(
            "finish to sync accumulator, its info is: {:?}",
            accumulator_info
        );

        Ok((self.start_leaf_index, self.accumulator))
    }
}
