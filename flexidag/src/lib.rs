pub mod flexidag_service;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use anyhow::bail;
pub use flexidag_service::FlexidagService;

use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::{ChainNetworkID, NodeConfig, RocksdbConfig};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig};
use starcoin_storage::Store;

pub fn try_init_with_storage(
    storage: Arc<dyn Store>,
    config: Arc<NodeConfig>,
) -> anyhow::Result<BlockDAG> {
    let dag = new_by_config(
        config.data_dir().join("flexidag").as_path(),
        config.net().id().clone(),
    )?;
    let startup_info = storage
        .get_startup_info()?
        .expect("startup info must exist");

    let block_header = storage
        .get_block_header_by_hash(startup_info.get_main().clone())?
        .expect("the genesis block in dag accumulator must none be none");
    let fork_height = block_header.dag_fork_height();
    match block_header.number().cmp(&fork_height) {
        std::cmp::Ordering::Greater | std::cmp::Ordering::Less => Ok(dag),
        std::cmp::Ordering::Equal => {
            // dag.commit(block_header)?;
            dag.init_with_genesis(block_header)?;
            Ok(dag)
        }
    }
}

pub fn new_by_config(db_path: &Path, _net: ChainNetworkID) -> anyhow::Result<BlockDAG> {
    let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
    let db = FlexiDagStorage::create_from_path(db_path, config)?;
    let dag = BlockDAG::new(8, db);
    Ok(dag)
}
