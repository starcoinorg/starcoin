use std::sync::{Arc, Mutex};

use anyhow::bail;
use starcoin_accumulator::Accumulator;
use starcoin_accumulator::{node::AccumulatorStoreType, MerkleAccumulator};
use starcoin_consensus::BlockDAG;
use starcoin_network_rpc_api::dag_protocol::{
    GetDagAccumulatorLeaves, GetTargetDagAccumulatorLeafDetail, RelationshipPair,
    TargetDagAccumulatorLeaf, TargetDagAccumulatorLeafDetail,
};
use starcoin_storage::storage::CodecKVStore;
use starcoin_storage::{flexi_dag::SyncFlexiDagSnapshotStorage, Store};

pub struct DagBlockChain {
    dag: Arc<Mutex<BlockDAG>>,
    dag_sync_accumulator: MerkleAccumulator,
    dag_sync_accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}

impl DagBlockChain {
    pub fn new(
        storage: Arc<dyn Store>,
        dag: Arc<Mutex<BlockDAG>>,
    ) -> anyhow::Result<Self> {
        // initialize the block accumulator
        let startup_info = match storage.get_flexi_dag_startup_info()? {
            Some(startup_info) => startup_info,
            None => {
                return Ok(Self {
                    dag: dag.clone(),
                    dag_sync_accumulator: MerkleAccumulator::new_empty(
                        storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
                    ),
                    dag_sync_accumulator_snapshot: storage.get_accumulator_snapshot_storage(),
                })
            }
        };

        // let accmulator_info = sync_flexi_dag_store.get_snapshot_storage().get(startup_info.main);
        let accumulator_info = match storage.query_by_hash(startup_info.main) {
            Ok(op_snapshot) => match op_snapshot {
                Some(snapshot) => snapshot.accumulator_info,
                None => bail!("failed to get sync accumulator info since it is None"),
            },
            Err(error) => bail!("failed to get sync accumulator info: {}", error.to_string()),
        };

        Ok(Self {
            dag: dag.clone(),
            dag_sync_accumulator: MerkleAccumulator::new_with_info(
                accumulator_info,
                storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
            ),
            dag_sync_accumulator_snapshot: storage.get_accumulator_snapshot_storage(),
        })
    }

    pub fn get_accumulator_leaves(
        &self,
        req: GetDagAccumulatorLeaves,
    ) -> anyhow::Result<Vec<TargetDagAccumulatorLeaf>> {
        match self
            .dag_sync_accumulator
            .get_leaves(req.accumulator_leaf_index, true, req.batch_size)
        {
            Ok(leaves) => Ok(leaves
                .into_iter()
                .enumerate()
                .map(
                    |(index, leaf)| match self.dag_sync_accumulator_snapshot.get(leaf) {
                        Ok(op_snapshot) => {
                            let snapshot = op_snapshot.expect("snapshot must exist");
                            TargetDagAccumulatorLeaf {
                                accumulator_root: snapshot.accumulator_info.accumulator_root,
                                leaf_index: req.accumulator_leaf_index.saturating_sub(index as u64),
                            }
                        }
                        Err(error) => {
                            panic!(
                                "error occured when query the accumulator snapshot: {}",
                                error.to_string()
                            );
                        }
                    },
                )
                .collect()),
            Err(error) => {
                bail!(
                    "an error occured when getting the leaves of the accumulator, {}",
                    error.to_string()
                );
            }
        }
    }

    pub fn get_target_dag_accumulator_leaf_detail(
        &self,
        req: GetTargetDagAccumulatorLeafDetail,
    ) -> anyhow::Result<Vec<TargetDagAccumulatorLeafDetail>> {
        let end_index = std::cmp::min(
            req.leaf_index + req.batch_size - 1,
            self.dag_sync_accumulator.get_info().num_leaves - 1,
        );
        let mut details = [].to_vec();
        for index in req.leaf_index..=end_index {
            let leaf_hash = self
                .dag_sync_accumulator
                .get_leaf(index)
                .unwrap_or(None)
                .expect("leaf hash should not be None");
            let snapshot = self
                .dag_sync_accumulator_snapshot
                .get(leaf_hash)
                .unwrap_or(None)
                .expect("the snapshot should not be None");
            let mut relationship_pair = [].to_vec();
            relationship_pair.extend(
                snapshot
                    .child_hashes
                    .into_iter()
                    .fold([].to_vec(), |mut pairs, child| {
                        let parents = self.dag.lock().expect("failed to lock the dag").get_parents(child).expect("a child must have parents");
                        parents.into_iter().for_each(|parent| {
                            pairs.push(RelationshipPair { parent, child });
                        });
                        pairs
                    })
                    .into_iter(),
            );

            details.push(TargetDagAccumulatorLeafDetail {
                accumulator_root: snapshot.accumulator_info.accumulator_root,
                relationship_pair,
            });
        }
        Ok(details)
    }
}
