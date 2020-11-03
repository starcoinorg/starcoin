// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::full_sync_task;
use crate::tasks::mock::SyncNodeMocker;
use anyhow::Result;
use logger::prelude::*;
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

    let node2 = SyncNodeMocker::new(net2, 1, 50)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();

    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        storage.clone(),
        node2,
        arc_node1.clone(),
    )?;
    let node2 = sync_task.await?;
    let current_block_header = node2.chain().current_header();

    assert_eq!(target.block_id, current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Arc::get_mut(&mut arc_node1).unwrap().produce_block(20)?;

    //sync again
    let target = arc_node1.chain().get_block_info(None)?.unwrap();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        storage.clone(),
        node2,
        arc_node1.clone(),
    )?;
    let node2 = sync_task.await?;
    let current_block_header = node2.chain().current_header();
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

    let node2 = SyncNodeMocker::new(net2, 1, 50)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();

    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        storage.clone(),
        node2,
        arc_node1.clone(),
    )?;
    let mut node2 = sync_task.await?;
    let current_block_header = node2.chain().current_header();

    assert_eq!(target.block_id, current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    //test fork

    Arc::get_mut(&mut arc_node1).unwrap().produce_block(10)?;
    node2.produce_block(10)?;

    let target = arc_node1.chain().get_block_info(None)?.unwrap();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        storage,
        node2,
        arc_node1.clone(),
    )?;
    let node2 = sync_task.await?;
    let current_block_header = node2.chain().current_header();
    assert_eq!(target.block_id, current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));
    Ok(())
}
