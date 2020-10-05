// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::time::Duration;

#[stest::test]
fn test_multi_client() -> Result<()> {
    let mut node_config = NodeConfig::random_for_test();
    node_config.miner.enable_miner_client = false;
    let config = Arc::new(node_config);
    let ws_address = config.rpc.ws_address.as_ref().unwrap();
    let ipc_file = config.rpc.get_ipc_file().to_path_buf();
    let url = format!("ws://{}", ws_address.to_string());
    debug!("url:{}", url);
    debug!("data_dir:{:?}", config.data_dir());

    let node_handle = test_helper::run_node_by_config(config)?;

    let rpc_service_ref = node_handle.rpc_service()?;
    let mut rt = tokio_compat::runtime::Runtime::new()?;

    std::thread::sleep(Duration::from_millis(300));

    let local_client = RpcClient::connect_local(rpc_service_ref)?;
    let status0 = local_client.node_status()?;
    info!("local_client status: {}", status0);

    let ipc_client = RpcClient::connect_ipc(ipc_file, &mut rt).expect("connect ipc fail.");
    let status1 = ipc_client.node_status()?;
    info!("ipc_client status: {}", status1);

    let ws_client =
        RpcClient::connect_websocket(url.as_str(), &mut rt).expect("connect websocket fail.");
    let status = ws_client.node_status()?;
    info!("ws_client node_status: {}", status);
    local_client.close();
    ipc_client.close();
    ws_client.close();
    if let Err(e) = node_handle.stop() {
        error!("node stop error: {:?}", e)
    }
    Ok(())
}
