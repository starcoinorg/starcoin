use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    u64,
};

use anyhow::Ok;
use parking_lot::RwLock;
use starcoin_crypto::HashValue;
use starcoin_dag::{
    blockdag::BlockDAG, consensusdb::{
        schema::{KeyCodec, ValueCodec},
        schemadb::MemoryReachabilityStore,
    }, reachability::inquirer, types::interval::Interval
};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockBody, BlockHeader, BlockHeaderBuilder, BlockHeaderExtra, BlockNumber},
    genesis_config::ChainId,
    transaction::{authenticator::AuthenticationKey, SignedUserTransaction},
    U256,
};

use crate::store::sync_absent_ancestor::DagSyncBlockKey;

use super::{
    sync_absent_ancestor::{AbsentDagBlockStoreReader, AbsentDagBlockStoreWriter, DagSyncBlock},
    sync_dag_store::SyncDagStore,
};

fn build_body_with_uncles(uncles: Vec<BlockHeader>) -> BlockBody {
    BlockBody::new(vec![SignedUserTransaction::mock()], Some(uncles))
}

fn build_version_0_block_header(body: HashValue, number: BlockNumber) -> BlockHeader {
    BlockHeaderBuilder::new()
        .with_parent_hash(HashValue::random())
        .with_timestamp(rand::random())
        .with_number(number)
        .with_author(AccountAddress::random())
        .with_author_auth_key(Some(AuthenticationKey::random()))
        .with_accumulator_root(HashValue::random())
        .with_parent_block_accumulator_root(HashValue::random())
        .with_state_root(HashValue::random())
        .with_gas_used(rand::random())
        .with_difficulty(U256::from(rand::random::<u64>()))
        .with_body_hash(body)
        .with_chain_id(ChainId::vega())
        .with_nonce(rand::random())
        .with_extra(BlockHeaderExtra::new([
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>(),
        ]))
        .with_parents_hash(vec![
            HashValue::random(),
            HashValue::random(),
            HashValue::random(),
            HashValue::random(),
        ])
        .with_version(0)
        .with_pruning_point(HashValue::zero())
        .build()
}

fn build_version_0_block(number: BlockNumber) -> Block {
    let body_without_uncle1 = build_body_with_uncles(vec![]);
    let body_without_uncle2 = build_body_with_uncles(vec![]);
    let header_without_uncle1 =
        build_version_0_block_header(body_without_uncle1.hash(), rand::random());
    let header_without_uncle2 =
        build_version_0_block_header(body_without_uncle2.hash(), rand::random());

    let body = build_body_with_uncles(vec![header_without_uncle1, header_without_uncle2]);
    let header = build_version_0_block_header(body.hash(), number);

    Block::new(header, body)
}

#[test]
fn test_add_reachability_data() -> anyhow::Result<()> {
    let mut sync_dag_store = SyncDagStore::create_for_testing()?;
    let reachability_store = sync_dag_store.reachability_store.clone();

    let mut writer = reachability_store.write();

    let x = HashValue::random();
    let a = HashValue::random();
    let b = HashValue::random();
    let c = HashValue::random();
    let d = HashValue::random();
    let e = HashValue::random();

    inquirer::init_with_params(writer.deref_mut(), x, Interval::maximal())?;

    inquirer::add_block(writer.deref_mut(), a, x, &mut [x].into_iter())?;
    inquirer::add_block(writer.deref_mut(), b, a, &mut [a].into_iter())?;
    inquirer::add_block(writer.deref_mut(), c, a, &mut [a].into_iter())?;
    inquirer::add_block(writer.deref_mut(), d, a, &mut [a].into_iter())?;
    inquirer::add_block(writer.deref_mut(), e, b, &mut [c, d].into_iter())?;

    drop(writer);

    let reader = reachability_store.read();

    assert!(inquirer::is_dag_ancestor_of(reader.deref(), a, b)?);
    assert!(inquirer::is_dag_ancestor_of(reader.deref(), a, c)?);
    assert!(inquirer::is_dag_ancestor_of(reader.deref(), a, d)?);
    assert!(inquirer::is_dag_ancestor_of(reader.deref(), a, e)?);
    assert!(inquirer::is_dag_ancestor_of(reader.deref(), b, e)?);
    assert!(inquirer::is_dag_ancestor_of(reader.deref(), c, e)?);
    assert!(inquirer::is_dag_ancestor_of(reader.deref(), d, e)?);

    drop(reader);

    anyhow::Ok(())
}

#[test]
fn test_sync_dag_absent_store() -> anyhow::Result<()> {
    let dag = BlockDAG::create_for_testing()?;
    let sync_dag_store = SyncDagStore::create_for_testing()?;

    // write and read
    let one = DagSyncBlock {
        block: Some(build_version_0_block(rand::random())),
    };

    sync_dag_store
        .absent_dag_store
        .save_absent_block(vec![one.clone()])?;
    let read_one = sync_dag_store.absent_dag_store.get_absent_block_by_id(
        one.block.as_ref().unwrap().header().number(),
        one.block.as_ref().unwrap().header().id(),
    )?;
    assert_eq!(one, read_one);

    // update
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
        block: Some(build_version_0_block(rand::random())),
    };
    let three = DagSyncBlock {
        block: Some(build_version_0_block(rand::random())),
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
    let dag = BlockDAG::create_for_testing()?;
    let sync_dag_store = SyncDagStore::create_for_testing()?;

    // write and read
    let one = build_version_0_block(1);

    // // write and read
    let two = build_version_0_block(2);

    // write and read
    let three = build_version_0_block(3);

    // write and read
    let four = build_version_0_block(4);

    // write and read
    let two_again = build_version_0_block(2);

    sync_dag_store.save_block(one.clone())?;
    sync_dag_store.save_block(two.clone())?;
    sync_dag_store.save_block(three.clone())?;
    sync_dag_store.save_block(four.clone())?;
    sync_dag_store.save_block(two_again.clone())?;

    let mut iter = sync_dag_store
        .iter_at_first()?
        .take(10)
        .collect::<Vec<_>>()
        .into_iter();

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
                    let key = DagSyncBlockKey::decode_key(&id_raw)?;
                    let dag_sync_block = DagSyncBlock::decode_value(&data_raw)?;
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

    let mut iter_to_see_empty = sync_dag_store.iter_at_first()?;
    assert!(iter_to_see_empty.next().is_some());

    sync_dag_store.delete_all_dag_sync_block()?;

    iter_to_see_empty = sync_dag_store.iter_at_first()?;
    assert!(iter_to_see_empty.next().is_none());

    Ok(())
}
