// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::HashValue;
use logger::prelude::*;
use network_api::messages::RawRpcRequestMessage;
use network_api::PeerProvider;
use network_p2p::Multiaddr;
use network_rpc_core::RawRpcClient;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::mocker::mock;
use starcoin_service_registry::{RegistryAsyncService, ServiceContext};
use std::any::Any;
use test_helper::build_network;
use test_helper::network::MockPeerMessageHandler;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TestRequest {
    pub data: HashValue,
}

pub fn mock_rpc_handler(
    req: Box<dyn Any>,
    _ctx: &mut ServiceContext<NetworkRpcService>,
) -> Box<dyn Any> {
    let mut req = req.downcast::<RawRpcRequestMessage>().unwrap();
    req.responder.try_send(req.request.2).unwrap();
    Box::new(())
}

#[stest::test]
async fn test_network_raw_rpc() {
    use std::time::Duration;

    let mocker1 = mock(mock_rpc_handler);
    let mock_message_handler1 = MockPeerMessageHandler::default();
    let (network1, node_config1, .., registry1) =
        build_network(None, Some(mocker1), mock_message_handler1)
            .await
            .unwrap();

    let seed: Multiaddr = node_config1.network.self_address().unwrap();

    let mock_message_handler2 = MockPeerMessageHandler::default();
    let mocker2 = mock(mock_rpc_handler);
    let (network2, _node_config2, .., registry2) =
        build_network(Some(seed), Some(mocker2), mock_message_handler2)
            .await
            .unwrap();

    let request = TestRequest {
        data: HashValue::random(),
    };
    //request from network2 -> network1
    let request = request.encode().unwrap();
    info!("req :{:?}", request);
    let resp = network2
        .send_raw_request(
            Some(network1.identify().clone()),
            "test".to_string(),
            request.clone(),
            Duration::from_secs(1),
        )
        .await;
    assert_eq!(request, resp.unwrap());

    // request from network1 -> network2
    let request = TestRequest {
        data: HashValue::random(),
    };

    let request = request.encode().unwrap();
    info!("req :{:?}", request);
    let resp = network1
        .send_raw_request(
            Some(network2.identify().clone()),
            "test".to_string(),
            request.clone(),
            Duration::from_secs(1),
        )
        .await;
    assert_eq!(request, resp.unwrap());

    registry2.shutdown_system().await.unwrap();
    registry1.shutdown_system().await.unwrap();
}
