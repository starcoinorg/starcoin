use crate::{
    block_connector::{BlockConnectorService, CheckBlockConnectorHashValue},
    tasks::full_sync_task,
};
use std::sync::Arc;

use super::mock::SyncNodeMocker;
use anyhow::{format_err, Result};
use futures::channel::mpsc::unbounded;
use starcoin_chain_api::ChainReader;
use starcoin_config::genesis_config::G_TEST_DAG_FORK_STATE_KEY;
use starcoin_dag::consensusdb::consenses_state::DagState;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_txpool_mock_service::MockTxPoolService;
use test_helper::DummyNetworkService;

async fn sync_block_process(
    target_node: Arc<SyncNodeMocker>,
    local_node: Arc<SyncNodeMocker>,
    registry: &ServiceRef<RegistryService>,
) -> Result<(Arc<SyncNodeMocker>, Arc<SyncNodeMocker>)> {
    loop {
        let target = target_node.sync_target();

        let storage = local_node.chain().get_storage();
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let current_block_id = startup_info.main;

        let local_net = local_node.chain_mocker.net();
        let (local_ancestor_sender, _local_ancestor_receiver) = unbounded();

        let block_chain_service = async_std::task::block_on(
            registry.service_ref::<BlockConnectorService<MockTxPoolService>>(),
        )?;
        let dag_fork_height = local_node.chain().dag_fork_height()?.unwrap_or(u64::MAX);

        let (sync_task, _task_handle, task_event_counter) = full_sync_task(
            current_block_id,
            target.clone(),
            false,
            local_net.time_service(),
            storage.clone(),
            block_chain_service,
            target_node.clone(),
            local_ancestor_sender,
            DummyNetworkService::default(),
            15,
            None,
            None,
            Some(dag_fork_height),
            local_node.chain().dag().clone(),
        )?;
        let branch = sync_task.await?;
        info!("checking branch in sync service is the same as target's branch");
        assert_eq!(branch.current_header().id(), target.target_id.id());

        let block_connector_service = registry
            .service_ref::<BlockConnectorService<MockTxPoolService>>()
            .await?
            .clone();
        let result = block_connector_service
            .send(CheckBlockConnectorHashValue {
                head_hash: target.target_id.id(),
                number: target.target_id.number(),
            })
            .await?;
        if result.is_ok() {
            break;
        }
        let reports = task_event_counter.get_reports();
        reports
            .iter()
            .for_each(|report| debug!("reports: {}", report));
    }

    Ok((local_node, target_node))
}

#[stest::test(timeout = 600)]
async fn test_sync_dag_blocks() -> Result<()> {
    let test_system = super::test_tools::SyncTestSystem::initialize_sync_system()
        .await
        .expect("failed to init system");

    test_system
        .target_node
        .chain()
        .dag()
        .save_dag_state(*G_TEST_DAG_FORK_STATE_KEY, DagState { tips: vec![] })?;
    test_system
        .local_node
        .chain()
        .dag()
        .save_dag_state(*G_TEST_DAG_FORK_STATE_KEY, DagState { tips: vec![] })?;

    let count = 10;

    let mut target_node = Arc::new(test_system.target_node);
    let local_node = Arc::new(test_system.local_node);
    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_block(count)
        .expect("failed to produce block");
    let dag_genesis_header = target_node.chain().status().head;
    assert!(
        dag_genesis_header.number() == count,
        "dag genesis header number should be {}, but {}",
        count,
        dag_genesis_header.number()
    );

    // sync, the local and target will be a single chain to be a dag chain
    let (local_node, mut target_node) =
        sync_block_process(target_node, local_node, &test_system.registry).await?;

    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_fork_chain(20, 25)?;

    Arc::get_mut(&mut target_node).unwrap().produce_block(3)?;

    sync_block_process(target_node, local_node, &test_system.registry).await?;

    Ok(())
}
