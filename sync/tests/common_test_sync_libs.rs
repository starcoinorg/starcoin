use anyhow::{Ok, Result};
use futures::executor::block_on;
use network_api::PeerId;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainAsyncService, ChainReader};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{genesis_config::G_TEST_DAG_FORK_STATE_KEY, *};
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::consenses_state::DagState};
use starcoin_logger::prelude::*;
use starcoin_miner::MinedBlock;
use starcoin_node::NodeHandle;
use starcoin_service_registry::{
    bus::{Bus, BusService},
    RegistryAsyncService, RegistryService, ServiceRef,
};
use starcoin_storage::Storage;
use starcoin_types::block::{BlockHeader, BlockNumber};
use starcoin_vm_types::on_chain_config::FlexiDagConfig;
use std::{sync::Arc, time::Duration};
use test_helper::Account;

#[derive(Debug, Clone)]
pub struct DagBlockInfo {
    pub header: BlockHeader,
    pub children: Vec<HashValue>,
}

// fixme: remove unused
#[allow(unused)]
pub fn gen_chain_env(config: NodeConfig) -> Result<NodeHandle> {
    test_helper::run_node_by_config(Arc::new(config))
}

// fixme: remove unused
#[allow(unused)]
fn gen_node(seeds: Vec<NetworkConfig>) -> Result<(NodeHandle, NetworkConfig)> {
    let dir = match temp_dir() {
        starcoin_config::DataDirPath::PathBuf(path) => path,
        starcoin_config::DataDirPath::TempPath(path) => path.path().to_path_buf(),
    };
    let mut config = NodeConfig::random_for_test_disable_miner(true);
    let net_addr = config.network.self_address();
    debug!("Local node address: {:?}", net_addr);

    config.network.seeds = seeds
        .into_iter()
        .map(|other_network_config| other_network_config.self_address())
        .collect::<Vec<_>>()
        .into();
    let network_config = config.network.clone();
    let handle = test_helper::run_node_by_config(Arc::new(config))?;

    Ok((handle, network_config))
}

// fixme: remove unused
#[allow(unused)]
pub fn init_multiple_node(count: usize) -> Result<Vec<NodeHandle>> {
    let mut result = vec![];
    result.reserve(count);
    let (main_node, network_config) = gen_node(vec![])?;
    result.push(main_node);
    for _ in 1..count {
        result.push(gen_node(vec![network_config.clone()])?.0);
    }
    Ok(result)
}

#[allow(unused)]
pub fn generate_block(handle: &NodeHandle, count: usize) -> Result<()> {
    for _i in 0..count {
        let _ = handle.generate_block()?;
    }
    Ok(())
}

#[allow(unused)]
pub fn generate_dag_fork_number(handle: &NodeHandle) -> Result<()> {
    // for _i in 0..G_TEST_DAG_FORK_HEIGHT - 3 {
    //     let (_block, _is_dag) = handle.generate_block()?;
    // }

    block_on(async move {
        let current_header = handle
            .registry()
            .service_ref::<ChainReaderService>()
            .await?
            .main_head_header()
            .await?;
        // let block_info = handle.storage().get_block_info(current_header.id())?.expect("failed to get the block info");

        // let accumulator = MerkleAccumulator::new_with_info(block_info.block_accumulator_info, handle.storage().get_accumulator_store(AccumulatorStoreType::Block));
        // let dag_genesis = accumulator.get_leaf(G_TEST_DAG_FORK_HEIGHT)?.expect("failed to get the dag genesis");
        // let dag_genesis_header = handle.storage().get_block(dag_genesis)?.expect("failed to get the dag genesis header");
        let mut dag = handle.registry().get_shared::<BlockDAG>().await?;
        // dag.init_with_genesis(dag_genesis_header.header().clone()).expect("failed to initialize dag");
        // Ok(())
        dag.save_dag_state(*G_TEST_DAG_FORK_STATE_KEY, DagState { tips: vec![] })
    })
}

#[allow(unused)]
pub fn generate_dag_block(handle: &NodeHandle, count: usize) -> Result<Vec<DagBlockInfo>> {
    let mut result = vec![];
    let dag = handle.get_dag()?;
    while result.len() < count {
        let (block, is_dag) = handle.generate_block()?;
        if is_dag {
            result.push(block);
        }
    }
    Ok(result
        .into_iter()
        .map(|block| DagBlockInfo {
            header: block.header().clone(),
            children: dag.get_children(block.header().id()).unwrap(),
        })
        .collect::<Vec<DagBlockInfo>>())
}

// fixme: remove unused
#[allow(unused)]
pub fn init_two_node() -> Result<(NodeHandle, NodeHandle, PeerId)> {
    // network1 initialization
    let (local_handle, local_net_addr) = {
        let local_config = NodeConfig::random_for_test();
        let net_addr = local_config.network.self_address();
        debug!("Local node address: {:?}", net_addr);
        (gen_chain_env(local_config).unwrap(), net_addr)
    };

    // network2 initialization
    let (target_handle, target_peer_id) = {
        let mut target_config = NodeConfig::random_for_test();
        target_config.network.seeds = vec![local_net_addr].into();
        let target_peer_id = target_config.network.self_peer_id();
        (gen_chain_env(target_config).unwrap(), target_peer_id)
    };
    Ok((local_handle, target_handle, target_peer_id))
}

/// Just for test
#[allow(unused)]
pub fn execute_dag_poll_block(
    registry: ServiceRef<RegistryService>,
    fork_number: BlockNumber,
) -> Result<u64> {
    let timestamp = block_on(async move {
        let node_config = registry
            .get_shared::<Arc<NodeConfig>>()
            .await
            .expect("Failed to get node config");
        let time_service = node_config.net().time_service();
        let chain_service = registry
            .service_ref::<ChainReaderService>()
            .await
            .expect("failed to get chain reader service");
        let header_hash = chain_service
            .main_head_header()
            .await
            .expect("failed to get header hash")
            .id();
        let storage = registry
            .get_shared::<Arc<Storage>>()
            .await
            .expect("failed to get storage");
        let dag = registry
            .get_shared::<BlockDAG>()
            .await
            .expect("failed to get dag");
        let mut chain = BlockChain::new(time_service, header_hash, storage, None, dag)
            .expect("failed to get new the chain");
        let net = node_config.net();
        let current_number = chain.status().head().number();
        chain = test_helper::dao::modify_on_chain_config_by_dao_block(
            Account::new(),
            chain,
            net,
            test_helper::dao::vote_flexi_dag_config(net, fork_number),
            test_helper::dao::on_chain_config_type_tag(FlexiDagConfig::type_tag()),
            test_helper::dao::execute_script_on_chain_config(net, FlexiDagConfig::type_tag(), 0u64),
        )
        .expect("failed to execute script for poll");

        let bus = registry
            .service_ref::<BusService>()
            .await
            .expect("failed to get bus service");
        // broadcast poll blocks
        for block_number in current_number + 1..=chain.status().head().number() {
            let block = chain
                .get_block_by_number(block_number)
                .expect("failed to get block by number")
                .unwrap();
            let block_info = chain
                .get_block_info(Some(block.id()))
                .expect("failed to get block info")
                .unwrap();
            bus.broadcast(MinedBlock(Arc::new(block)))
                .expect("failed to broadcast new head block");
        }

        loop {
            if chain_service
                .main_head_block()
                .await
                .expect("failed to get main head block")
                .header()
                .number()
                == chain.status().head().number()
            {
                break;
            } else {
                async_std::task::sleep(Duration::from_millis(500)).await;
            }
        }
        chain.time_service().now_millis()
    });
    Ok(timestamp)
}
