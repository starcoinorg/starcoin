use std::sync::Arc;

use anyhow::bail;
use dag_consensus::blockdag::BlockDAG;
use dag_database::prelude::{FlexiDagStorage, FlexiDagStorageConfig};
use starcoin_accumulator::{node::AccumulatorStoreType, MerkleAccumulator};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_storage::{flexi_dag::SyncFlexiDagSnapshotStorage, Store};
use starcoin_types::block::BlockHeader;
use starcoin_types::{blockhash::ORIGIN, header::Header};

pub struct DagBlockChain {
    dag: Option<BlockDAG>,
    dag_sync_accumulator: MerkleAccumulator,
    sync_accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}

impl DagBlockChain {
    pub fn new(
        config: Arc<NodeConfig>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
    ) -> anyhow::Result<Self> {
        // initialize the dag
        let db_path = config.storage.dir();
        let config = FlexiDagStorageConfig::create_with_params(1, 0, 1024);
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = BlockDAG::new(
            Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]),
            16,
            db,
        );

        // initialize the block accumulator
        let startup_info = match storage.get_flexi_dag_startup_info()? {
            Some(startup_info) => startup_info,
            None => {
                return Ok(Self {
                    dag: Some(dag),
                    dag_sync_accumulator: MerkleAccumulator::new_empty(
                        storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
                    ),
                    sync_accumulator_snapshot: storage.get_accumulator_snapshot_storage(),
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
            dag: Some(dag),
            dag_sync_accumulator: MerkleAccumulator::new_with_info(
                accumulator_info,
                storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
            ),
            sync_accumulator_snapshot: storage.get_accumulator_snapshot_storage(),
        })
    }
}
