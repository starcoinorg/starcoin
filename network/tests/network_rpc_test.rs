// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures_timer::Delay;
use logger::prelude::*;
use network_api::PeerProvider;
use network_p2p_types::ProtocolRequest;
use network_rpc_core::RawRpcClient;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{RegistryAsyncService, ServiceContext};
use starcoin_types::peer_info::RpcInfo;
use std::any::Any;
use test_helper::build_network;
use test_helper::network::MockPeerMessageHandler;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TestRequest {
    pub data: HashValue,
}

pub struct MockRpcHandler;

impl MockHandler<NetworkRpcService> for MockRpcHandler {
    fn handle(
        &mut self,
        _r: Box<dyn Any>,
        _ctx: &mut ServiceContext<NetworkRpcService>,
    ) -> Box<dyn Any> {
        unreachable!()
    }

    fn handle_event(&mut self, msg: Box<dyn Any>, _ctx: &mut ServiceContext<NetworkRpcService>) {
        let req = msg.downcast::<ProtocolRequest>().unwrap();
        req.request
            .pending_response
            .send(req.request.payload)
            .unwrap();
    }
}

#[stest::test]
async fn test_network_raw_rpc() {
    use std::time::Duration;
    let rpc_info = RpcInfo::new(vec!["test".to_string()]);

    let (network1, _mock_message_handler1, node_config1, .., registry1) =
        build_network(None, (rpc_info.clone(), MockRpcHandler))
            .await
            .unwrap();
    let peer_id_1 = node_config1.network.self_peer_id().unwrap();
    let seed = node_config1.network.self_address().unwrap();

    let (network2, _mock_message_handler2, node_config2, .., registry2) =
        build_network(Some(seed), (rpc_info, MockRpcHandler))
            .await
            .unwrap();
    let peer_id_2 = node_config2.network.self_peer_id().unwrap();
    Delay::new(Duration::from_secs(1)).await;
    let request = TestRequest {
        data: HashValue::random(),
    };
    //request from network2 -> network1
    let request = request.encode().unwrap();
    info!("req :{:?}", request);
    let resp = network2
        .send_raw_request(peer_id_1, "test".to_string(), request.clone())
        .await;
    assert_eq!(request, resp.unwrap());

    // request from network1 -> network2
    let request = TestRequest {
        data: HashValue::random(),
    };

    let request = request.encode().unwrap();
    info!("req :{:?}", request);
    let resp = network1
        .send_raw_request(peer_id_2, "test".to_string(), request.clone())
        .await;
    assert_eq!(request, resp.unwrap());

    registry2.shutdown_system().await.unwrap();
    registry1.shutdown_system().await.unwrap();
}
