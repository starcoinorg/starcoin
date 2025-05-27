use std::sync::Arc;

use starcoin_config::temp_dir;
use starcoin_crypto::HashValue as Hash;
use starcoin_dag::{
    consensusdb::{
        prelude::{FlexiDagStorage, FlexiDagStorageConfig},
        schemadb::DbReachabilityStore,
    },
    reachability::tests::{DagBlock, DagBuilder},
};
use starcoin_types::{
    block::{BlockHeader, BlockHeaderBuilder, BlockNumber},
    blockhash::BlockLevel,
    consensus_header::{ConsensusHeader, HeaderWithBlockLevel},
};

fn create_block_with_level(
    pruning_point: Hash,
    selected_parent: Hash,
    parents: Vec<Vec<Hash>>,
    level: BlockLevel,
    number: BlockNumber,
) -> HeaderWithBlockLevel {
    let header_builder = BlockHeaderBuilder::random();
    let header = header_builder
        .with_parent_hash(selected_parent)
        .with_parents_hash(parents)
        .with_number(number)
        .with_pruning_point(pruning_point)
        .build();

    HeaderWithBlockLevel {
        header: Arc::new(header),
        block_level: level,
    }
}

#[test]
fn test_parents_builder() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let k = 3;
    let max_block_level = 5;
    let cache_size = 1;

    let config = FlexiDagStorageConfig {
        cache_size,
        ..Default::default()
    };

    let db = FlexiDagStorage::create_db(temp_dir(), config)?;

    let mut reachability_store = DbReachabilityStore::new(db.clone(), cache_size);
    // let mut relation_store = MemoryRelationsStore::new();
    // let reachability_service = dag.reachability_service().clone();
    // let relations_service = MTRelationsService::new(dag.storage.relations_store.clone(), 0);

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    // let mut parents_manager = ParentsManager::new(max_block_level, genesis.id(), headers_store.clone(), reachability_service.clone(), relations_service.clone());
    let mut dag_builder = DagBuilder::new(&mut reachability_store);

    // dag.init_with_genesis(genesis.clone()).unwrap();
    let pruning_point = create_block_with_level(
        genesis.id(),
        genesis.id(),
        vec![
            vec![genesis.id()],
            vec![1001.into()],
            vec![1001.into()],
            vec![1001.into()],
            vec![1001.into()],
        ],
        0,
        1,
    );

    let header2 = create_block_with_level(
        pruning_point.header.id(),
        pruning_point.header.id(),
        vec![
            vec![pruning_point.header.id()],
            vec![1001.into()],
            vec![1001.into()],
            vec![1001.into()],
            vec![1001.into()],
        ],
        0,
        2,
    );

    dag_builder
        .init(genesis.parent_hash())
        .add_block(DagBlock {
            hash: genesis.id(),
            parents: vec![genesis.parent_hash()],
        })
        .add_block(DagBlock {
            hash: pruning_point.header.id(),
            parents: pruning_point.header.parents().first().unwrap().clone(),
        })
        .add_block(DagBlock {
            hash: header2.header.id(),
            parents: header2.header.parents().first().unwrap().clone(),
        });

    Ok(())
}
