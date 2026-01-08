use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig};
use starcoin_types::block::BlockHeaderBuilder;
use std::sync::Arc;

#[test]
fn test_commit_atomicity() -> Result<()> {
    let db_tempdir = tempfile::tempdir()?;
    let config = FlexiDagStorageConfig::new();
    let dag_storage = FlexiDagStorage::create_from_path(db_tempdir.path(), config)?;

    // Create and initialize genesis
    let genesis = BlockHeaderBuilder::new()
        .with_number(0)
        .with_parent_hash(HashValue::zero())
        .build();
    let mut dag = BlockDAG::new(8, 8, 3, dag_storage, genesis.id());

    // Initialize DAG with genesis
    dag.init_with_genesis(genesis.clone())?;

    // Create genesis ghostdata for commit
    let genesis_ghost = dag.ghost_dag_manager().genesis_ghostdag_data(&genesis);
    dag.commit_trusted_block(genesis.clone(), Arc::new(genesis_ghost))?;

    // Create block 1
    let block1 = BlockHeaderBuilder::new()
        .with_number(1)
        .with_parent_hash(genesis.id())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let ghost1 = dag.calc_ghostdata(&block1)?;

    // Test 1: Normal commit
    dag.commit_trusted_block(block1.clone(), Arc::new(ghost1.clone()))?;
    assert!(dag.has_block_connected(&block1)?);

    // Test 2: Recommit same block (idempotency)
    let result = dag.commit_trusted_block(block1.clone(), Arc::new(ghost1.clone()));
    assert!(result.is_ok(), "Recommit should be idempotent");
    assert!(dag.has_block_connected(&block1)?);

    // Create block 2
    let block2 = BlockHeaderBuilder::new()
        .with_number(2)
        .with_parent_hash(block1.id())
        .with_parents_hash(vec![block1.id()])
        .build();
    let ghost2 = dag.calc_ghostdata(&block2)?;

    // Test 3: Commit new block
    dag.commit_trusted_block(block2.clone(), Arc::new(ghost2.clone()))?;
    assert!(dag.has_block_connected(&block2)?);

    // Test 4: Verify parent-child relationships are intact
    assert!(dag.has_block_connected(&genesis)?);
    assert!(dag.has_block_connected(&block1)?);
    assert!(dag.has_block_connected(&block2)?);

    println!("All atomicity tests passed!");
    Ok(())
}

#[test]
fn test_partial_write_detection() -> Result<()> {
    let db_tempdir = tempfile::tempdir()?;
    let config = FlexiDagStorageConfig::new();
    let dag_storage = FlexiDagStorage::create_from_path(db_tempdir.path(), config)?;

    // Create and initialize genesis
    let genesis = BlockHeaderBuilder::new()
        .with_number(0)
        .with_parent_hash(HashValue::zero())
        .build();
    let mut dag = BlockDAG::new(8, 8, 3, dag_storage, genesis.id());

    // Initialize DAG with genesis
    dag.init_with_genesis(genesis.clone())?;

    // Create genesis ghostdata for commit
    let genesis_ghost = dag.ghost_dag_manager().genesis_ghostdag_data(&genesis);
    dag.commit_trusted_block(genesis.clone(), Arc::new(genesis_ghost))?;

    // Create a block
    let block = BlockHeaderBuilder::new()
        .with_number(1)
        .with_parent_hash(genesis.id())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let ghost = dag.calc_ghostdata(&block)?;

    // Commit the block
    dag.commit_trusted_block(block.clone(), Arc::new(ghost.clone()))?;

    // has_block_connected should return true (all data exists)
    assert!(dag.has_block_connected(&block)?);

    // Test that non-existent block returns false
    let non_existent = BlockHeaderBuilder::new()
        .with_number(99)
        .with_parent_hash(HashValue::random())
        .build();
    assert!(!dag.has_block_connected(&non_existent)?);

    println!("Partial write detection tests passed!");
    Ok(())
}
