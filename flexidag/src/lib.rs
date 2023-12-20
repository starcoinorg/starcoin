pub mod flexidag_service;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use anyhow::bail;
pub use flexidag_service::FlexidagService;
pub mod dag_accumulator_controller;
pub use dag_accumulator_controller::DagAccumulatorController;

use bcs_ext::BCSCodec;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::{ChainNetworkID, NodeConfig, RocksdbConfig};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig};
use starcoin_storage::flexi_dag::SyncFlexiDagSnapshotHasher;
use starcoin_storage::Store;
use starcoin_types::dag_block::KTotalDifficulty;
use starcoin_types::startup_info::DagStartupInfo;

pub fn try_init_with_storage(
    storage: Arc<dyn Store>,
    config: Arc<NodeConfig>,
) -> anyhow::Result<(Option<BlockDAG>, Option<MerkleAccumulator>)> {
    let startup_info = storage
        .get_startup_info()?
        .expect("startup info must exist");
    if storage.get_dag_startup_info()?.is_some() {
        let accumulator_info = storage
            .get_dag_accumulator_info()?
            .expect("dag accumulator info should exist");
        assert!(
            accumulator_info.get_num_leaves() > 0,
            "the number of dag accumulator leaf must be greater than 0"
        );
        let dag_accumulator = MerkleAccumulator::new_with_info(
            accumulator_info,
            storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
        );

        Ok((
            Some(new_by_config(
                config.data_dir().join("flexidag").as_path(),
                config.net().id().clone(),
            )?),
            Some(dag_accumulator),
        ))
    } else {
        let block_header = storage
            .get_block_header_by_hash(startup_info.get_main().clone())?
            .expect("the genesis block in dag accumulator must none be none");
        let fork_height = block_header.dag_fork_height();
        match block_header.number().cmp(&fork_height) {
            std::cmp::Ordering::Less => Ok((None, None)),
            std::cmp::Ordering::Equal => {
                let dag_accumulator = MerkleAccumulator::new_with_info(
                    AccumulatorInfo::default(),
                    storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
                );

                let mut k_total_difficulties = BTreeSet::new();
                k_total_difficulties.insert(KTotalDifficulty {
                    head_block_id: block_header.id(),
                    total_difficulty: storage
                        .get_block_info(block_header.id())?
                        .expect("block info must exist")
                        .get_total_difficulty(),
                });
                let snapshot_hasher = SyncFlexiDagSnapshotHasher {
                    child_hashes: vec![block_header.id()],
                    head_block_id: block_header.id(),
                    k_total_difficulties,
                };
                let key = calculate_dag_accumulator_key(&snapshot_hasher)?;
                dag_accumulator.append(&[key])?;
                storage.put_hashes(key, snapshot_hasher.to_snapshot(dag_accumulator.get_info()))?;
                storage.save_dag_startup_info(DagStartupInfo::new(key))?;
                dag_accumulator.flush()?;
                let dag = new_by_config(
                    config.data_dir().join("flexidag").as_path(),
                    config.net().id().clone(),
                )?;
                // dag.commit(block_header)?;
                dag.init_with_genesis(block_header)?;
                Ok((Some(dag), Some(dag_accumulator)))
            }
            std::cmp::Ordering::Greater => {
                bail!("failed to init dag")
            }
        }
    }
}
pub fn calculate_dag_accumulator_key(
    snapshot: &SyncFlexiDagSnapshotHasher,
) -> anyhow::Result<HashValue> {
    Ok(HashValue::sha3_256_of(&snapshot.encode().expect(
        "encoding the sorted relatship set must be successful",
    )))
}
pub fn new_by_config(db_path: &Path, _net: ChainNetworkID) -> anyhow::Result<BlockDAG> {
    let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
    let db = FlexiDagStorage::create_from_path(db_path, config)?;
    let dag = BlockDAG::new(8, db);
    Ok(dag)
}
