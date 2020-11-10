// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::full_sync_task;
use crate::tasks::mock::SyncNodeMocker;
use anyhow::Result;
use futures::channel::mpsc::unbounded;
use futures_timer::Delay;
use logger::prelude::*;
use pin_utils::core_reexport::time::Duration;
use starcoin_chain_api::ChainReader;
use starcoin_vm_types::genesis_config::{BuiltinNetworkID, ChainNetwork};
use std::sync::Arc;

#[stest::test]
pub async fn test_full_sync_new_node() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.block_id);
    assert_eq!(target.block_id, current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Arc::get_mut(&mut arc_node1).unwrap().produce_block(20)?;

    let (sender, receiver) = unbounded();
    //sync again
    let target = arc_node1.chain().get_block_info(None)?.unwrap();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.block_id);
    assert_eq!(target.block_id, current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[stest::test]
pub async fn test_full_sync_fork() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let mut node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.block_id);
    assert_eq!(target.block_id, current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    //test fork

    Arc::get_mut(&mut arc_node1).unwrap().produce_block(10)?;
    node2.produce_block(5)?;

    let (sender, receiver) = unbounded();
    let target = arc_node1.chain().get_block_info(None)?.unwrap();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage,
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.block_id);
    assert_eq!(target.block_id, current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));
    Ok(())
}

#[stest::test]
pub async fn test_full_sync_fork_from_genesis() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    //fork from genesis
    let mut node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;
    node2.produce_block(5)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.block_id);
    assert_eq!(target.block_id, current_block_header.id());
    assert_eq!(
        arc_node1.chain().current_header().id(),
        current_block_header.id()
    );
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[stest::test]
pub async fn test_full_sync_continue() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 10, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    //fork from genesis
    let mut node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;
    node2.produce_block(7)?;

    // first set target to 5.
    let target_block = arc_node1.chain().get_block_by_number(5)?.unwrap();
    let target = arc_node1
        .chain()
        .get_block_info(Some(target_block.id()))?
        .unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;

    assert_eq!(branch.current_header().id(), target.block_id);
    let current_block_header = node2.chain().current_header();
    // node2's main chain not change.
    assert_ne!(target.block_id, current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("task_report: {}", report));

    //set target to latest.
    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let (sender, receiver) = unbounded();
    //continue sync
    //TODO find a way to verify continue sync will reuse previous task local block.
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;

    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.block_id);
    assert_eq!(target.block_id, current_block_header.id());
    assert_eq!(
        arc_node1.chain().current_header().id(),
        current_block_header.id()
    );
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[stest::test]
pub async fn test_full_sync_cancel() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 10, 50)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sync_task, task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let sync_join_handle = tokio::task::spawn(sync_task);

    Delay::new(Duration::from_millis(100)).await;

    task_handle.cancel();
    let sync_result = sync_join_handle.await?;
    assert!(sync_result.is_err());
    assert!(sync_result.err().unwrap().is_canceled());

    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_ne!(target.block_id, current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[ignore]
#[stest::test]
pub async fn test_full_sync_by_total_difficulty() {
    //TODO add a test to verify low block number but high total difficulty.
}
