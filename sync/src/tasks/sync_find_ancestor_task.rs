use anyhow::{format_err, Result};
use futures::{future::BoxFuture, FutureExt};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_network_rpc_api::dag_protocol::TargetDagAccumulatorLeaf;
use starcoin_storage::{flexi_dag::SyncFlexiDagSnapshotStorage, storage::CodecKVStore};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

use super::sync_dag_protocol_trait::PeerSynDagAccumulator;

#[derive(Clone)]
pub struct FindAncestorTask {
    start_leaf_number: u64,
    fetcher: Arc<dyn PeerSynDagAccumulator>,
    batch_size: u64,
}
impl FindAncestorTask {
    pub(crate) fn new<F>(current_leaf_numeber: u64, target_leaf_numeber: u64, fetcher: F) -> Self
    where
        F: PeerSynDagAccumulator + 'static,
    {
        FindAncestorTask {
            start_leaf_number: std::cmp::min(current_leaf_numeber, target_leaf_numeber),
            fetcher: Arc::new(fetcher),
            batch_size: 3,
        }
    }
}

impl TaskState for FindAncestorTask {
    type Item = TargetDagAccumulatorLeaf;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let current_number = self.start_leaf_number;
            let target_accumulator_leaves = self
                .fetcher
                .get_sync_dag_asccumulator_leaves(None, self.start_leaf_number, self.batch_size)
                .await?;
            Ok(target_accumulator_leaves)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        //this should never happen, because all node's genesis block should same.
        if self.start_leaf_number == 0 {
            return None;
        }

        let next_number = self.start_leaf_number.saturating_sub(self.batch_size);
        Some(Self {
            start_leaf_number: next_number,
            batch_size: self.batch_size,
            fetcher: self.fetcher.clone(),
        })
    }
}

pub struct AncestorCollector {
    accumulator: Arc<MerkleAccumulator>,
    ancestor: Option<AccumulatorInfo>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}

impl AncestorCollector {
    pub fn new(
        accumulator: Arc<MerkleAccumulator>,
        accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    ) -> Self {
        Self {
            accumulator,
            ancestor: None,
            accumulator_snapshot,
        }
    }
}

impl TaskResultCollector<TargetDagAccumulatorLeaf> for AncestorCollector {
    type Output = AccumulatorInfo;

    fn collect(&mut self, item: TargetDagAccumulatorLeaf) -> anyhow::Result<CollectorState> {
        if self.ancestor.is_some() {
            return Ok(CollectorState::Enough);
        }

        let accumulator_leaf = self.accumulator.get_leaf(item.leaf_index)?.ok_or_else(|| {
            format_err!(
                "Cannot find accumulator leaf by number: {}",
                item.leaf_index
            )
        })?;

        let accumulator_info = match self.accumulator_snapshot.get(accumulator_leaf)? {
            Some(snapshot) => snapshot.accumulator_info,
            None => panic!("failed to get the snapshot, it is none."),
        };

        if item.accumulator_root == accumulator_info.accumulator_root {
            self.ancestor = Some(accumulator_info);
            return anyhow::Result::Ok(CollectorState::Enough);
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(mut self) -> Result<Self::Output> {
        self.ancestor
            .take()
            .ok_or_else(|| format_err!("Unexpect state, collector finished by ancestor is None"))
    }
}
