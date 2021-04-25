// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::BCSCodec;
use futures_timer::Delay;
use network_rpc_core::RawRpcClient;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_service_registry::RegistryAsyncService;
use starcoin_types::peer_info::RpcInfo;
use std::borrow::Cow;
use test_helper::build_network;
use test_helper::network::MockRpcHandler;
#[cfg(test)]
mod network_service_test;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TestRequest {
    pub data: HashValue,
}

#[stest::test]
async fn test_network_raw_rpc() {
    use std::time::Duration;
    let rpc_info = RpcInfo::new(vec!["test"]);

    let service1 = build_network(None, Some((rpc_info.clone(), MockRpcHandler::echo())))
        .await
        .unwrap();
    let peer_id_1 = service1.config.network.self_peer_id();
    let seed = service1.config.network.self_address();

    let service2 = build_network(Some(seed), Some((rpc_info, MockRpcHandler::echo())))
        .await
        .unwrap();
    let peer_id_2 = service2.config.network.self_peer_id();
    Delay::new(Duration::from_secs(1)).await;
    let request = TestRequest {
        data: HashValue::random(),
    };
    //request from network2 -> network1
    let request = request.encode().unwrap();
    info!("req :{:?}", request);
    let resp = service2
        .service_ref
        .send_raw_request(peer_id_1, Cow::from("test"), request.clone())
        .await;
    assert_eq!(request, resp.unwrap());

    // request from network1 -> network2
    let request = TestRequest {
        data: HashValue::random(),
    };

    let request = request.encode().unwrap();
    info!("req :{:?}", request);
    let resp = service1
        .service_ref
        .send_raw_request(peer_id_2, Cow::from("test"), request.clone())
        .await;
    assert_eq!(request, resp.unwrap());

    service2.registry.shutdown_system().await.unwrap();
    service1.registry.shutdown_system().await.unwrap();
}
