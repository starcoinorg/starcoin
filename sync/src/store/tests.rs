use anyhow::Ok;
use bcs_ext::{BCSCodec, Sample};
use starcoin_types::block::{Block, BlockBody, BlockHeaderBuilder};

use crate::store::sync_absent_ancestor::DagSyncBlockKey;

use super::{
    sync_absent_ancestor::{AbsentDagBlockStoreReader, AbsentDagBlockStoreWriter, DagSyncBlock},
    sync_dag_store::SyncDagStore,
};

#[test]
fn test_sync_dag_absent_store() -> anyhow::Result<()> {
    let sync_dag_store = SyncDagStore::create_for_testing()?;

    // write and read
    let one = DagSyncBlock {
        block: Some(Block::random()),
        children: vec![0.into()],
    };
    sync_dag_store
        .absent_dag_store
        .save_absent_block(vec![one.clone()])?;
    let mut read_one = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        one.block.as_ref().unwrap().header().number(),
        one.block.as_ref().unwrap().header().id(),
    )?;
    assert_eq!(one, read_one);

    // update
    read_one.children.push(1.into());
    sync_dag_store
        .absent_dag_store
        .save_absent_block(vec![read_one.clone()])?;
    let read_one_again = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        one.block.as_ref().unwrap().header().number(),
        one.block.as_ref().unwrap().header().id(),
    )?;
    assert_eq!(read_one, read_one_again);

    // delete
    sync_dag_store.absent_dag_store.delete_absent_block(
        one.block.as_ref().unwrap().header().number(),
        one.block.as_ref().unwrap().header().id(),
    )?;
    let deleted_one = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        one.block.as_ref().unwrap().header().number(),
        one.block.as_ref().unwrap().header().id(),
    );
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
    let read_one_rewrite = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        read_one.block.as_ref().unwrap().header().number(),
        read_one.block.as_ref().unwrap().header().id(),
    )?;
    let read_two = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        two.block.as_ref().unwrap().header().number(),
        two.block.as_ref().unwrap().header().id(),
    )?;
    let read_three = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        three.block.as_ref().unwrap().header().number(),
        three.block.as_ref().unwrap().header().id(),
    )?;
    assert_eq!(read_one_rewrite, one);
    assert_eq!(two, read_two);
    assert_eq!(three, read_three);

    Ok(())
}

#[test]
fn test_write_read_in_order() -> anyhow::Result<()> {
    let sync_dag_store = SyncDagStore::create_for_testing()?;

    // write and read
    let one = Block::new(
        BlockHeaderBuilder::new()
            .with_number(1)
            .with_parents_hash(Some(vec![]))
            .build(),
        BlockBody::sample(),
    );

    // // write and read
    let two = Block::new(
        BlockHeaderBuilder::new()
            .with_number(2)
            .with_nonce(109)
            .with_parents_hash(Some(vec![]))
            .build(),
        BlockBody::sample(),
    );

    // write and read
    let three = Block::new(
        BlockHeaderBuilder::new()
            .with_number(3)
            .with_parents_hash(Some(vec![]))
            .build(),
        BlockBody::sample(),
    );

    // write and read
    let four = Block::new(
        BlockHeaderBuilder::new()
            .with_number(4)
            .with_parents_hash(Some(vec![]))
            .build(),
        BlockBody::sample(),
    );

    // write and read
    let two_again = Block::new(
        BlockHeaderBuilder::new()
            .with_number(2)
            .with_nonce(101)
            .with_parents_hash(Some(vec![]))
            .build(),
        BlockBody::sample(),
    );

    sync_dag_store.save_block(one.clone())?;
    sync_dag_store.save_block(two.clone())?;
    sync_dag_store.save_block(three.clone())?;
    sync_dag_store.save_block(four.clone())?;
    sync_dag_store.save_block(two_again.clone())?;

    sync_dag_store.update_children(one.header().number(), one.id(), two.id())?;
    sync_dag_store.update_children(two.header().number(), two.id(), three.id())?;
    sync_dag_store.update_children(three.header().number(), three.id(), four.id())?;
    sync_dag_store.update_children(one.header().number(), one.id(), two_again.id())?;

    let mut iter = sync_dag_store.iter_at_first()?;

    let mut expect_order = vec![one, two, three, four, two_again];
    expect_order.sort_by(|first, second| {
        if first.header().number() != second.header().number() {
            first.header().number().cmp(&second.header().number())
        } else {
            first.id().cmp(&second.id())
        }
    });

    let mut db_order = vec![];

    loop {
        let mut op_next = iter.next();
        if op_next.is_none() {
            break;
        }

        while let Some(next) = op_next {
            match next {
                std::result::Result::Ok((id_raw, data_raw)) => {
                    let key = DagSyncBlockKey::decode(&id_raw)?;
                    let dag_sync_block = DagSyncBlock::decode(&data_raw)?;
                    println!(
                        "id: {:?}, id in data: {:?}, number: {:?}",
                        key,
                        dag_sync_block.block.as_ref().unwrap().id(),
                        dag_sync_block.block.as_ref().unwrap().header().number()
                    );
                    db_order.push(dag_sync_block.block.unwrap().clone());
                }
                Err(e) => {
                    println!("error: {:?}", e);
                    return Err(e);
                }
            }
            op_next = iter.next();
        }
    }

    assert_eq!(expect_order, db_order);

    Ok(())
}
