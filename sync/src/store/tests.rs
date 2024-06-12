use anyhow::Ok;
use starcoin_types::block::Block;

use super::{
    sync_absent_ancestor::{AbsentDagBlockStoreReader, AbsentDagBlockStoreWriter, DagSyncBlock},
    sync_dag_store::SyncDagStore,
};

#[test]
fn test_sync_dag_absent_store() -> anyhow::Result<()> {
    let mut sync_dag_store = SyncDagStore::create_for_testing()?;

    // write and read
    let one = DagSyncBlock {
        block: Some(Block::random()),
        children: vec![0.into()],
    };
    sync_dag_store
        .absent_dag_store
        .save_absent_block(vec![one.clone()])?;
    let mut read_one = sync_dag_store
        .absent_dag_store
        .get_absent_block_by_id(one.block.as_ref().unwrap().header().id())?;
    assert_eq!(one, read_one);

    // update
    read_one.children.push(1.into());
    sync_dag_store
        .absent_dag_store
        .save_absent_block(vec![read_one.clone()])?;
    let read_one_again = sync_dag_store
        .absent_dag_store
        .get_absent_block_by_id(one.block.as_ref().unwrap().header().id())?;
    assert_eq!(read_one, read_one_again);

    // delete
    sync_dag_store
        .absent_dag_store
        .delete_absent_block(one.block.as_ref().unwrap().header().id())?;
    let deleted_one = sync_dag_store
        .absent_dag_store
        .get_absent_block_by_id(one.block.as_ref().unwrap().header().id());
    assert!(deleted_one.is_err());
    println!("read a deleted syn dag block return: {:?}", deleted_one);

    // append new absent blocks
    let two = DagSyncBlock {
        block: Some(Block::random()),
        children: vec![2.into(), 3.into()],
    };
    let three = DagSyncBlock {
        block: Some(Block::random()),
        children: vec![4.into(), 5.into()],
    };
    sync_dag_store.absent_dag_store.save_absent_block(vec![
        one.clone(),
        two.clone(),
        three.clone(),
    ])?;
    let read_one_rewrite = sync_dag_store
        .absent_dag_store
        .get_absent_block_by_id(read_one.block.as_ref().unwrap().header().id())?;
    let read_two = sync_dag_store
        .absent_dag_store
        .get_absent_block_by_id(two.block.as_ref().unwrap().header().id())?;
    let read_three = sync_dag_store
        .absent_dag_store
        .get_absent_block_by_id(three.block.as_ref().unwrap().header().id())?;
    assert_eq!(read_one_rewrite, one);
    assert_eq!(two, read_two);
    assert_eq!(three, read_three);

    Ok(())
}
