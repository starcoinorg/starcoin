
use forkable_jellyfish_merkle::node_type::Node;
use starcoin_config::*;
use starcoin_node::NodeHandle;
use std::sync::Arc;
use network_api::PeerId;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeader;
use anyhow::{Ok, Result};
use starcoin_logger::prelude::*;

#[derive(Debug, Clone)]
pub struct DagBlockInfo {
    pub header: BlockHeader,
    pub children: Vec<HashValue>,
}

pub fn gen_chain_env(config: NodeConfig) -> Result<NodeHandle> {
    test_helper::run_node_by_config(Arc::new(config))
}

fn gen_node(seeds: Vec<NetworkConfig>) -> Result<(NodeHandle, NetworkConfig)> {
    let dir = match temp_dir() {
        starcoin_config::DataDirPath::PathBuf(path) => path,
        starcoin_config::DataDirPath::TempPath(path) => {
            path.path().to_path_buf()
        }
    };
    let mut config = NodeConfig::proxima_for_test(dir);
    let net_addr = config.network.self_address();
    debug!("Local node address: {:?}", net_addr);

    config.network.seeds = seeds.into_iter().map(|other_network_config| {
        other_network_config.self_address()
    }).collect::<Vec<_>>().into();
    let network_config = config.network.clone();
    let handle = test_helper::run_node_by_config(Arc::new(config))?;

    Ok((handle, network_config))
}

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

pub fn generate_dag_block(handle: &NodeHandle, count: usize) -> Result<Vec<DagBlockInfo>> {
    let mut result = vec![];
    let dag = handle.get_dag()?;
    while result.len() < count {
        let block = handle.generate_block()?;
        if block.header().is_dag() {
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