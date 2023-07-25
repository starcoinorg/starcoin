use std::sync::Arc;

use consensus::blockdag::BlockDAG;
use consensus_types::{blockhash::ORIGIN, header::Header};
use database::prelude::{FlexiDagStorageConfig, FlexiDagStorage};
use starcoin_accumulator::MerkleAccumulator;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_storage::{flexi_dag::{SyncFlexiDagSnapshotStorage, SyncFlexiDagStorage}, Store};
use starcoin_types::{block::BlockHeader, dag_block::DagBlockHeader};



pub struct DagBlockChain {
    dag: Option<BlockDAG>,
    dag_block_accumulator: MerkleAccumulator,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}


impl DagBlockChain {
    pub fn new(
        config: Arc<NodeConfig>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
    ) -> anyhow::Result<Self> {
        todo!()
        // initialize the dag
        // let db_path = config.storage.dir();
        // let config = FlexiDagStorageConfig::create_with_params(1, 0, 1024);
        // let db = FlexiDagStorage::create_from_path(db_path, config)?;
        // let dag = BlockDAG::new(Header::new(DagBlockHeader::random(), vec![HashValue::new(ORIGIN)]), 16, db);

        // // initialize the block accumulator
        // let sync_flexi_dag_store = Arc::new(SyncFlexiDagStorage::new(storage,)?);
        // let startup_info = match storage.get_flexi_dag_startup_info()? {
        //     Some(startup_info) => startup_info,
        //     None => {
        //         return Ok(Self {
        //             dag: Some(dag),
        //             dag_block_accumulator: MerkleAccumulator::new_empty(sy),
        //             accumulator_snapshot: sync_flexi_dag_store.get_accumulator_storage(),
        //         })
        //     }
        // };

        // let accmulator_info = sync_flexi_dag_store.get_snapshot_storage().get(startup_info.main);
 
        // Ok(Self {
        //     dag: Some(dag),
        //     dag_block_accumulator: MerkleAccumulator::new_with_info(accmulator_info, sync_flexi_dag_store.get_accumulator_storage()),
        //     accumulator_snapshot: Arc::new(SyncFlexiDagSnapshotStorage::new(
        //         storage,
        //     )?),
        // })
    }


}