// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use starcoin_types::system_events::MintBlockEvent;
use std::sync::Arc;
use std::time::Duration;

//TODO fixme
#[ignore]
#[stest::test]
async fn test_in_async() -> Result<()> {
    do_client_test()
}

fn do_client_test() -> Result<()> {
    let node_config = NodeConfig::random_for_test();
    let config = Arc::new(node_config);
    let ipc_file = config.rpc.get_ipc_file();
    let url = config.rpc.get_ws_address().unwrap();
    debug!("url:{}", url);
    debug!("data_dir:{:?}", config.data_dir());

    let node_handle = test_helper::run_node_by_config(config)?;

    let rpc_service_ref = node_handle.rpc_service()?;

    std::thread::sleep(Duration::from_millis(300));

    let local_client = RpcClient::connect_local(rpc_service_ref)?;
    let status0 = local_client.node_info()?;
    info!("local_client status: {:?}", status0);

    let ipc_client = RpcClient::connect_ipc(ipc_file).expect("connect ipc fail.");
    let status1 = ipc_client.node_info()?;
    info!("ipc_client status: {:?}", status1);

    let ws_client =
        RpcClient::connect_websocket(url.to_string().as_str()).expect("connect websocket fail.");
    let status = ws_client.node_info()?;
    info!("ws_client node_status: {:?}", status);
    local_client.close();
    ipc_client.close();
    ws_client.close();
    if let Err(e) = node_handle.stop() {
        error!("node stop error: {:?}", e)
    }
    Ok(())
}

#[stest::test]
fn test_multi_client() -> Result<()> {
    do_client_test()
}

#[stest::test(timeout = 120)]
fn test_client_reconnect() -> Result<()> {
    let mut node_config = NodeConfig::random_for_test();
    node_config.miner.disable_miner_client = Some(false);
    let config = Arc::new(node_config);
    let url = config.rpc.get_ws_address().unwrap();
    debug!("url:{}", url);
    debug!("data_dir:{:?}", config.data_dir());

    let node_handle = test_helper::run_node_by_config(config.clone())?;
    std::thread::sleep(Duration::from_millis(300));

    let ws_client =
        RpcClient::connect_websocket(url.to_string().as_str()).expect("connect websocket fail.");
    let status = ws_client.node_info()?;
    info!("ws_client node_status: {:?}", status);

    let _e = node_handle.stop();

    let node_handle = test_helper::run_node_by_config(config)?;
    std::thread::sleep(Duration::from_millis(300));
    //first call after lost connection will return error
    let result = ws_client.node_info();
    assert!(result.is_err());
    //second call will return ok
    let result = ws_client.node_info();
    assert!(result.is_ok());

    info!("ws_client node_status: {:?}", result.unwrap());

    let _e = node_handle.stop();
    Ok(())
}

#[stest::test(timeout = 120)]
fn test_client_reconnect_subscribe() -> Result<()> {
    let node_config = NodeConfig::random_for_test();
    let config = Arc::new(node_config);
    let url = config.rpc.get_ws_address().unwrap();
    debug!("url:{}", url);
    debug!("data_dir:{:?}", config.data_dir());

    let node_handle = test_helper::run_node_by_config(config.clone())?;
    std::thread::sleep(Duration::from_millis(300));

    let ws_client =
        RpcClient::connect_websocket(url.to_string().as_str()).expect("connect websocket fail.");
    let stream1 = ws_client.subscribe_new_mint_blocks()?;
    let handle1 = async_std::task::spawn(async move {
        stream1
            .into_stream()
            .collect::<Vec<Result<MintBlockEvent>>>()
            .await
    });
    node_handle.generate_block()?;
    std::thread::sleep(Duration::from_millis(300));
    let _e = node_handle.stop();

    let node_handle = test_helper::run_node_by_config(config)?;
    std::thread::sleep(Duration::from_millis(300));
    //first call after lost connection will return error
    let result = ws_client.node_info();
    assert!(result.is_err());

    let stream2 = ws_client.subscribe_new_mint_blocks()?;
    let handle2 = async_std::task::spawn(async move {
        stream2
            .into_stream()
            .collect::<Vec<Result<MintBlockEvent>>>()
            .await
    });

    node_handle.generate_block()?;
    std::thread::sleep(Duration::from_millis(300));
    let _e = node_handle.stop();

    let events1 = futures::executor::block_on(async move { handle1.await });
    let events2 = futures::executor::block_on(async move { handle2.await });
    assert_ne!(events1.len(), 0);
    assert_ne!(events2.len(), 0);
    Ok(())
}
