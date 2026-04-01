// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use jsonrpsee::core::client::SubscriptionClientT;
use jsonrpsee::rpc_params;
use jsonrpsee_ws_client::WsClientBuilder;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::pubsub::Kind;
use starcoin_rpc_client::RpcClient;
use starcoin_types::system_events::MintBlockEvent;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

static CLIENT_SERVER_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

//TODO fixme
#[ignore]
#[stest::test]
async fn test_in_async() -> Result<()> {
    do_client_test()
}

fn do_client_test() -> Result<()> {
    let _guard = CLIENT_SERVER_TEST_LOCK.lock().expect("lock test guard");
    let node_config = NodeConfig::random_for_test();
    let config = Arc::new(node_config);
    let ipc_file = config.rpc.get_ipc_file();
    let http_url = format!(
        "http://127.0.0.1:{}",
        config.rpc.get_http_address().unwrap().port
    );
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

    let http_client = RpcClient::connect_http(http_url.as_str()).expect("connect http fail.");
    let status_http = wait_for_rpc_ready(Duration::from_secs(10), || http_client.node_info())?;
    info!("http_client status: {:?}", status_http);
    assert!(
        http_client.subscribe_new_blocks().is_err(),
        "http/https transport must not support pubsub"
    );

    let ws_client =
        RpcClient::connect_websocket(url.to_string().as_str()).expect("connect websocket fail.");
    let status = ws_client.node_info()?;
    info!("ws_client node_status: {:?}", status);
    local_client.close();
    ipc_client.close();
    http_client.close();
    ws_client.close();
    if let Err(e) = node_handle.stop() {
        error!("node stop error: {:?}", e)
    }
    Ok(())
}

fn wait_for_rpc_ready<T, F>(timeout: Duration, mut f: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let deadline = Instant::now() + timeout;
    let mut last_err = None;
    loop {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) if Instant::now() < deadline => {
                last_err = Some(err);
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(err) => return Err(last_err.unwrap_or(err)),
        }
    }
}

#[stest::test]
fn test_multi_client() -> Result<()> {
    do_client_test()
}

#[stest::test(timeout = 120)]
fn test_client_reconnect() -> Result<()> {
    let _guard = CLIENT_SERVER_TEST_LOCK.lock().expect("lock test guard");
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

    // wait for the threads that are minting and executing blocks to exit
    std::thread::sleep(Duration::from_millis(10000));

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
    let _guard = CLIENT_SERVER_TEST_LOCK.lock().expect("lock test guard");
    let rt = tokio::runtime::Runtime::new().unwrap();
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
    let handle1 = rt.spawn(async move {
        stream1
            .into_stream()
            .collect::<Vec<Result<MintBlockEvent>>>()
            .await
    });
    std::thread::sleep(Duration::from_millis(500));
    node_handle.generate_block()?;
    std::thread::sleep(Duration::from_millis(300));
    node_handle.generate_block()?;
    std::thread::sleep(Duration::from_millis(300));
    let _e = node_handle.stop();

    let node_handle = test_helper::run_node_by_config(config)?;
    std::thread::sleep(Duration::from_millis(300));
    //first call after lost connection will return error
    let result = ws_client.node_info();
    assert!(result.is_err());

    let stream2 = ws_client.subscribe_new_mint_blocks()?;
    let handle2 = rt.spawn(async move {
        stream2
            .into_stream()
            .collect::<Vec<Result<MintBlockEvent>>>()
            .await
    });

    std::thread::sleep(Duration::from_millis(500));
    node_handle.generate_block()?;
    std::thread::sleep(Duration::from_millis(300));
    node_handle.generate_block()?;
    std::thread::sleep(Duration::from_millis(300));
    let _e = node_handle.stop();

    let events1 = futures::executor::block_on(handle1)?;
    let events2 = futures::executor::block_on(handle2)?;
    assert_ne!(events1.len(), 0);
    assert_ne!(events2.len(), 0);
    Ok(())
}

#[stest::test(timeout = 120)]
fn test_legacy_pubsub_subscribe_compat() -> Result<()> {
    let _guard = CLIENT_SERVER_TEST_LOCK.lock().expect("lock test guard");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let node_config = NodeConfig::random_for_test();
    let config = Arc::new(node_config);
    let url = config.rpc.get_ws_address().unwrap();
    let node_handle = test_helper::run_node_by_config(config)?;
    std::thread::sleep(Duration::from_millis(300));

    let ws = rt.block_on(async { WsClientBuilder::default().build(url.to_string()).await })?;
    let mut sub = rt.block_on(async {
        ws.subscribe::<serde_json::Value, _>(
            "starcoin_subscribe",
            rpc_params![vec![Kind::NewHeads]],
            "starcoin_unsubscribe",
        )
        .await
    })?;

    std::thread::sleep(Duration::from_millis(500));
    node_handle.generate_block()?;
    let msg = rt
        .block_on(async { tokio::time::timeout(Duration::from_secs(10), sub.next()).await })?
        .expect("legacy subscription stream closed unexpectedly")?;
    assert!(
        msg.get("header").is_some(),
        "legacy subscription payload must contain block header"
    );

    let _e = node_handle.stop();
    Ok(())
}
