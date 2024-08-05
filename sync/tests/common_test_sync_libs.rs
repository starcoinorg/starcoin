use anyhow::{Ok, Result};
use starcoin_config::{temp_dir, NetworkConfig, NodeConfig};
use starcoin_logger::prelude::*;
use starcoin_node::NodeHandle;
use std::sync::Arc;

// fixme: remove unused
fn gen_node(seeds: Vec<NetworkConfig>) -> Result<(NodeHandle, NetworkConfig)> {
    let _dir = match temp_dir() {
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
    let mut result = Vec::with_capacity(count);
    let (main_node, network_config) = gen_node(vec![])?;
    result.push(main_node);
    for _ in 1..count {
        result.push(gen_node(vec![network_config.clone()])?.0);
    }
    Ok(result)
}
