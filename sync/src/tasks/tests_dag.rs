use crate::{
    block_connector::{BlockConnectorService, CheckBlockConnectorHashValue, CreateBlockRequest},
    tasks::full_sync_task,
};
use std::sync::Arc;

use super::mock::SyncNodeMocker;
use super::test_tools::full_sync_new_node;
use anyhow::{format_err, Result};
use futures::channel::mpsc::unbounded;
use starcoin_account_api::AccountInfo;
use starcoin_chain_api::{message::ChainResponse, ChainReader};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::genesis_config::{G_TEST_DAG_FORK_HEIGHT, G_TEST_DAG_FORK_STATE_KEY};
use starcoin_dag::consensusdb::consenses_state::DagState;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_txpool_mock_service::MockTxPoolService;
use test_helper::DummyNetworkService;

#[stest::test(timeout = 120)]
pub async fn test_full_sync_new_node_dag() -> Result<()> {
    starcoin_types::block::set_test_flexidag_fork_height(10);
    full_sync_new_node().await?;
    starcoin_types::block::reset_test_custom_fork_height();
    Ok(())
}

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

async fn sync_block_in_block_connection_service_mock(
    mut target_node: Arc<SyncNodeMocker>,
    local_node: Arc<SyncNodeMocker>,
    registry: &ServiceRef<RegistryService>,
    block_count: u64,
) -> Result<(Arc<SyncNodeMocker>, Arc<SyncNodeMocker>)> {
    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_block(block_count)?;
    sync_block_process(target_node, local_node, registry).await
}

#[stest::test(timeout = 600)]
async fn test_sync_single_chain_to_dag_chain() -> Result<()> {
    starcoin_types::block::set_test_flexidag_fork_height(10);
    let test_system = super::test_tools::SyncTestSystem::initialize_sync_system().await?;
    let (_local_node, _target_node) = sync_block_in_block_connection_service_mock(
        Arc::new(test_system.target_node),
        Arc::new(test_system.local_node),
        &test_system.registry,
        40,
    )
    .await?;
    starcoin_types::block::reset_test_custom_fork_height();
    Ok(())
}

#[stest::test(timeout = 600)]
async fn test_sync_red_blocks_dag() -> Result<()> {
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

    let mut target_node = Arc::new(test_system.target_node);
    let local_node = Arc::new(test_system.local_node);
    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_block(G_TEST_DAG_FORK_HEIGHT)
        .expect("failed to produce block");
    let dag_genesis_header = target_node.chain().status().head;
    assert!(
        dag_genesis_header.number() == G_TEST_DAG_FORK_HEIGHT,
        "dag genesis header number should be 10, but {}",
        dag_genesis_header.number()
    );

    // sync, the local and target will be a single chain to be a dag chain
    let (local_node, mut target_node) =
        sync_block_process(target_node, local_node, &test_system.registry).await?;

    // produce dag blocks in the local node
    // the blocks following the 10th block will be blue dag blocks
    let dag_block_count = 5;
    let block_connect_service = test_system
        .registry
        .service_ref::<BlockConnectorService<MockTxPoolService>>()
        .await?;
    let miner_info = AccountInfo::random();
    block_connect_service
        .send(CreateBlockRequest {
            count: dag_block_count,
            author: *miner_info.address(),
            parent_hash: None,
            user_txns: vec![],
            uncles: vec![],
            block_gas_limit: None,
            tips: None,
        })
        .await??;

    // wait for the dag block to be created
    async_std::task::sleep(std::time::Duration::from_secs(8)).await;

    let chain_reader_service = test_system
        .registry
        .service_ref::<ChainReaderService>()
        .await?;
    match chain_reader_service
        .send(starcoin_chain_api::message::ChainRequest::GetHeadChainStatus())
        .await??
    {
        ChainResponse::ChainStatus(chain_status) => {
            debug!(
                "local_node chain hash: {:?}, number: {:?}",
                chain_status.head.id(),
                chain_status.head.number()
            );
            assert_eq!(
                chain_status.head.number(),
                G_TEST_DAG_FORK_HEIGHT + dag_block_count
            );
        }
        _ => {
            panic!("failed to get chain status");
        }
    }

    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_fork_chain(20, 25)?;

    Arc::get_mut(&mut target_node).unwrap().produce_block(3)?;

    sync_block_process(target_node, local_node, &test_system.registry).await?;

    // // genertate the red blocks
    // Arc::get_mut(&mut target_node).unwrap().produce_block_by_header(dag_genesis_header, 5).expect("failed to produce block");

    Ok(())
}
