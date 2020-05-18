// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use actix::prelude::*;
use anyhow::Result;
use futures::channel::oneshot;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::node::NodeApi;
use starcoin_rpc_client::RpcClient;
use starcoin_rpc_server::module::NodeRpcImpl;
use starcoin_rpc_server::RpcActor;
use std::sync::Arc;
use std::time::Duration;

#[ignore]
#[test]
fn test_multi_client() -> Result<()> {
    starcoin_logger::init_for_test();
    let mut system = System::new("test");

    let config = Arc::new(NodeConfig::random_for_test());
    let ws_address = config.rpc.ws_address.as_ref().unwrap();
    let ipc_file = config.rpc.get_ipc_file().to_path_buf();
    let url = format!("ws://{}", ws_address.to_string());
    debug!("url:{}", url);
    debug!("data_dir:{:?}", config.data_dir());

    system.block_on(async {
        let (stop_sender, stop_receiver) = oneshot::channel::<bool>();
        //io_handler.add_method("status", |_params: Params| Ok(Value::Bool(true)));
        let (_rpc_actor, _) = RpcActor::launch_with_method(
            config.clone(),
            NodeRpcImpl::new(config, None).to_delegate(),
        )
        .unwrap();

        let client_task = move || {
            info!("client thread start.");
            std::thread::sleep(Duration::from_millis(300));

            let ws_client = RpcClient::connect_websocket(url.as_str()).unwrap();
            let status = ws_client.node_status().unwrap();
            info!("http_client status: {}", status);
            assert!(status);

            let ipc_client = RpcClient::connect_ipc(ipc_file).unwrap();
            let status1 = ipc_client.node_status().unwrap();
            info!("ipc_client status: {}", status1);
            assert_eq!(status, status1);

            // json_rpc's LocalRpc is not support middleware MetaIoHandler.
            // let local_client = RpcClient::connect_local(iohandler);
            // let status2 = local_client.node_status().unwrap();
            // info!("local_client status: {}", status2);
            // assert!(status2);

            drop(stop_sender);
        };

        let handle = std::thread::spawn(client_task);

        debug!("wait server stop");
        debug!("stop receiver: {}", stop_receiver.await.is_ok());
        handle.join().unwrap();
        debug!("server stop.");
    });

    Ok(())
}
