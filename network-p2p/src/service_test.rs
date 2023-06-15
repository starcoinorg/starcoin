// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::business_layer_handle::BusinessLayerHandle;
use crate::config::RequestResponseConfig;
use crate::protocol::{CustomMessageOutcome, rep};
use crate::protocol::generic_proto::NotificationsSink;
use crate::service::NetworkStateInfo;
use crate::{config, Event, NetworkService, NetworkWorker, GenericProtoOut};
use crate::{NetworkConfiguration, Params, ProtocolId};
use anyhow::{Ok, Result};
use bcs_ext::BCSCodec;
use futures::prelude::*;
use futures::stream::StreamExt;
use libp2p::PeerId;
use network_p2p_types::MultiaddrWithPeerId;
use once_cell::sync::Lazy;
use sc_peerset::{SetId, ReputationChange};
use serde::{Serialize, Deserialize};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::borrow::Cow;
use std::{sync::Arc, time::Duration};
use Event::NotificationStreamOpened;

static G_TEST_CHAIN_INFO: Lazy<Status> =
    Lazy::new(|| Status::default());

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Status {
    /// Protocol version.
    pub version: u32,
    /// Minimum supported version.
    pub min_supported_version: u32,
    /// Tell other peer which notification protocols we support.
    pub notif_protocols: Vec<Cow<'static, str>>,
    /// Tell other peer which rpc api we support.
    pub rpc_protocols: Vec<Cow<'static, str>>,
    /// the generic data related to the peer
    pub chain_info: ChainInfo,
}

impl std::default::Default for Status {
    fn default() -> Self {
        Self { 
            version: Default::default(), 
            min_supported_version: Default::default(), 
            notif_protocols: Default::default(), 
            rpc_protocols: Default::default(), 
            chain_info: ChainInfo::random() 
        }
    }
}

struct TestChainInfoHandle {
    status: Status,
}

impl TestChainInfoHandle {
    pub fn new(status: Status) -> Self {
        TestChainInfoHandle { status }
    }
}

impl BusinessLayerHandle for TestChainInfoHandle {
    fn handshake(&self, peer_id: PeerId, set_id: SetId, protocol_name: Cow<'static, str>, 
                received_handshake: Vec<u8>, notifications_sink: NotificationsSink) -> Result<CustomMessageOutcome, ReputationChange> {
        let status = Status::decode(&received_handshake).unwrap();
        if self.status.chain_info.genesis_hash() == status.chain_info.genesis_hash() {
            return std::result::Result::Ok(CustomMessageOutcome::NotificationStreamOpened {
                remote: peer_id,
                protocol: protocol_name,
                notifications_sink,
                generic_data: status.chain_info.encode().unwrap(),
                notif_protocols: status.notif_protocols,
                rpc_protocols: status.rpc_protocols,
            });
        }
        return Err(rep::BAD_MESSAGE);
}

    fn get_generic_data(&self) -> Result<Vec<u8>, anyhow::Error> {
        Ok(self.status.encode().unwrap())
    }

    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), anyhow::Error> {
        self.status = Status::decode(peer_info).unwrap();
        Ok(())
    }

    fn update_status(&mut self, peer_status: &[u8]) -> Result<(), anyhow::Error> {
        self.status.chain_info
            .update_status(ChainStatus::decode(peer_status).unwrap());
        Ok(())
    }

    fn build_handshake_msg(&mut self, notif_protocols: Vec<Cow<'static, str>>, rpc_protocols: Vec<Cow<'static, str>>) -> std::result::Result<Vec<u8>, anyhow::Error> {
        let status= Status {
            version: 1,
            min_supported_version: 1,
            notif_protocols,
            rpc_protocols,
            chain_info: ChainInfo::random(),
        };
        Ok(status.encode().unwrap())
    }
}

struct TestChainInfoHandle {
    status: Status,
}

impl TestChainInfoHandle {
    pub fn new(status: Status) -> Self {
        TestChainInfoHandle { status }
    }
}

impl BusinessLayerHandle for TestChainInfoHandle {
    fn handshake(&self, peer_id: PeerId, set_id: SetId, protocol_name: Cow<'static, str>, 
                received_handshake: Vec<u8>, notifications_sink: NotificationsSink) -> Result<CustomMessageOutcome, ReputationChange> {
        let status = Status::decode(&received_handshake).unwrap();
        if self.status.chain_info.genesis_hash() == status.chain_info.genesis_hash() {
            return std::result::Result::Ok(CustomMessageOutcome::NotificationStreamOpened {
                remote: peer_id,
                protocol: protocol_name,
                notifications_sink,
                generic_data: status.chain_info.encode().unwrap(),
                notif_protocols: status.notif_protocols,
                rpc_protocols: status.rpc_protocols,
            });
        }
        return Err(rep::BAD_MESSAGE);
}

    fn get_generic_data(&self) -> Result<Vec<u8>, anyhow::Error> {
        Ok(self.status.encode().unwrap())
    }

    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), anyhow::Error> {
        self.status = Status::decode(peer_info).unwrap();
        Ok(())
    }

    fn update_status(&mut self, peer_status: &[u8]) -> Result<(), anyhow::Error> {
        self.status.chain_info
            .update_status(ChainStatus::decode(peer_status).unwrap());
        Ok(())
    }

    fn build_handshake_msg(&mut self, notif_protocols: Vec<Cow<'static, str>>, rpc_protocols: Vec<Cow<'static, str>>) -> std::result::Result<Vec<u8>, anyhow::Error> {
        let status= Status {
            version: 1,
            min_supported_version: 1,
            notif_protocols,
            rpc_protocols,
            chain_info: ChainInfo::random(),
        };
        Ok(status.encode().unwrap())
    }
}

/// Builds a full node to be used for testing. Returns the node service and its associated events
/// stream.
///
/// > **Note**: We return the events stream in order to not possibly lose events between the
/// >   construction of the service and the moment the events stream is grabbed.
fn build_test_full_node(
    config: config::NetworkConfiguration,
) -> (Arc<NetworkService>, impl Stream<Item = Event>) {
    let worker = NetworkWorker::new(config::Params::<TestChainInfoHandle> {
        network_config: config,
        protocol_id: config::ProtocolId::from("/test-protocol-name"),
        business_layer_handle: TestChainInfoHandle::new(G_TEST_CHAIN_INFO.clone()),
        metrics_registry: None,
    })
    .unwrap();

    let service = worker.service().clone();
    let event_stream = service.event_stream("test");

    tokio::task::spawn(async move {
        futures::pin_mut!(worker);
        let _ = worker.await;
    });

    (service, event_stream)
}

/// Builds two nodes and their associated events stream.
/// The nodes are connected together and have the `ENGINE_ID` protocol registered.
fn build_nodes_one_proto() -> (
    Arc<NetworkService>,
    impl Stream<Item = Event>,
    Arc<NetworkService>,
    impl Stream<Item = Event>,
) {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];

    let (node1, events_stream1) = build_test_full_node(config::NetworkConfiguration {
        //notifications_protocols: vec![(ENGINE_ID, From::from("/foo"))],
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![listen_addr.clone()],
        transport: config::TransportConfig::MemoryOnly,
        ..config::NetworkConfiguration::new_local()
    });

    let (node2, events_stream2) = build_test_full_node(config::NetworkConfiguration {
        //notifications_protocols: vec![(ENGINE_ID, From::from("/foo"))],
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![],
        boot_nodes: vec![config::MultiaddrWithPeerId {
            multiaddr: listen_addr,
            peer_id: node1.local_peer_id(),
        }],
        transport: config::TransportConfig::MemoryOnly,
        ..config::NetworkConfiguration::new_local()
    });

    (node1, events_stream1, node2, events_stream2)
}

#[stest::test(timeout = 120)]
async fn lots_of_incoming_peers_works() {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];

    let (main_node, _) = build_test_full_node(config::NetworkConfiguration {
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![listen_addr.clone()],
        in_peers: u32::max_value(),
        transport: config::TransportConfig::MemoryOnly,
        ..config::NetworkConfiguration::new_local()
    });

    let main_node_peer_id = main_node.local_peer_id();

    // We spawn background tasks and push them in this `Vec`. They will all be waited upon before
    // this test ends.
    let mut background_tasks_to_wait = Vec::new();

    for _ in 0..32 {
        let (_dialing_node, event_stream) = build_test_full_node(config::NetworkConfiguration {
            notifications_protocols: vec![From::from(PROTOCOL_NAME)],
            listen_addresses: vec![],
            transport: config::TransportConfig::MemoryOnly,
            boot_nodes: vec![config::MultiaddrWithPeerId {
                multiaddr: listen_addr.clone(),
                peer_id: main_node_peer_id,
            }],
            ..config::NetworkConfiguration::new_local()
        });

        background_tasks_to_wait.push(async_std::task::spawn(async move {
            // Create a dummy timer that will "never" fire, and that will be overwritten when we
            // actually need the timer. Using an Option would be technically cleaner, but it would
            // make the code below way more complicated.
            let mut timer = futures_timer::Delay::new(Duration::from_secs(3600 * 24 * 7)).fuse();
            let mut event_stream = event_stream.fuse();
            loop {
                futures::select! {
                    _ = timer => {
                        // Test succeeds when timer fires.
                        return;
                    }
                    ev = event_stream.next() => {
                        match ev.unwrap() {
                            Event::NotificationStreamOpened { remote, .. } => {
                                assert_eq!(remote, main_node_peer_id);
                                // Test succeeds after 5 seconds. This timer is here in order to
                                // detect a potential problem after opening.
                                timer = futures_timer::Delay::new(Duration::from_secs(5)).fuse();
                            }
                            Event::NotificationStreamClosed { .. } => {
                                // Test failed.
                                panic!();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }));
    }
    future::join_all(background_tasks_to_wait).await;
}

#[stest::test(timeout = 600)]
async fn notifications_back_pressure() {
    // Node 1 floods node 2 with notifications. Random sleeps are done on node 2 to simulate the
    // node being busy. We make sure that all notifications are received.

    const TOTAL_NOTIFS: usize = 10_000;

    let (node1, mut events_stream1, node2, mut events_stream2) = build_nodes_one_proto();
    let node2_id = node2.local_peer_id();

    let receiver = tokio::task::spawn(async move {
        let mut received_notifications = 0;

        while received_notifications < TOTAL_NOTIFS {
            match events_stream2.next().await.unwrap() {
                Event::NotificationStreamClosed { .. } => panic!(),
                Event::NotificationsReceived { messages, .. } => {
                    for (protocol_name, message) in messages {
                        assert_eq!(protocol_name, PROTOCOL_NAME);
                        assert_eq!(message, format!("hello #{}", received_notifications));
                        received_notifications += 1;
                        debug!("received_notifications: {:?}", received_notifications)
                    }
                }
                _ => {}
            };

            if rand::random::<u8>() < 2 {
                tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 750)).await;
            }
        }
    });

    // Wait for the `NotificationStreamOpened`.
    loop {
        match events_stream1.next().await.unwrap() {
            NotificationStreamOpened { .. } => break,
            e => {
                debug!("receive event: {:?}", e);
            }
        };
    }
    debug!("Start sending..");
    for num in 0..TOTAL_NOTIFS {
        let notif = node1
            .notification_sender(node2_id, From::from(PROTOCOL_NAME))
            .unwrap();
        notif
            .ready()
            .await
            .unwrap()
            .send(format!("hello #{}", num))
            .unwrap();
    }
    receiver.await.unwrap();
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_listen_addresses_consistent_with_transport_memory() {
    let listen_addr = config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)];

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_listen_addresses_consistent_with_transport_not_memory() {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_boot_node_addresses_consistent_with_transport_memory() {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];
    let boot_node = config::MultiaddrWithPeerId {
        multiaddr: config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)],
        peer_id: PeerId::random(),
    };

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        boot_nodes: vec![boot_node],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_boot_node_addresses_consistent_with_transport_not_memory() {
    let listen_addr = config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)];
    let boot_node = config::MultiaddrWithPeerId {
        multiaddr: config::build_multiaddr![Memory(rand::random::<u64>())],
        peer_id: PeerId::random(),
    };

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        boot_nodes: vec![boot_node],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_reserved_node_addresses_consistent_with_transport_memory() {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];
    let boot_node = config::MultiaddrWithPeerId {
        multiaddr: config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)],
        peer_id: PeerId::random(),
    };

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        boot_nodes: vec![boot_node],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_reserved_node_addresses_consistent_with_transport_not_memory() {
    let listen_addr = config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)];
    let boot_node = config::MultiaddrWithPeerId {
        multiaddr: config::build_multiaddr![Memory(rand::random::<u64>())],
        peer_id: PeerId::random(),
    };

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        boot_nodes: vec![boot_node],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_public_addresses_consistent_with_transport_memory() {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];
    let public_address = config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)];

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        public_addresses: vec![public_address],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_public_addresses_consistent_with_transport_not_memory() {
    let listen_addr = config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)];
    let public_address = config::build_multiaddr![Memory(rand::random::<u64>())];

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        public_addresses: vec![public_address],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

const PROTOCOL_NAME: &str = "/starcoin/notify/1";

// //FIXME
// #[stest::test(timeout = 5)]
// #[allow(clippy::string_lit_as_bytes)]
// #[ignore]
// fn test_notify() {
//     let protocol = ProtocolId::from(b"stargate".as_ref());
//     let config1 = generate_config(vec![]);
//
//     let worker1 = NetworkWorker::new(Params::new(config1.clone(), protocol.clone())).unwrap();
//     let service1 = worker1.service().clone();
//     let mut stream1 = service1.event_stream();
//
//     task::spawn(worker1);
//
//     let addr1_hex = service1.peer_id().to_base58();
//     let seed: Multiaddr = format!(
//         "{}/p2p/{}",
//         &config1.listen_addresses.get(0).expect("should have"),
//         addr1_hex
//     )
//     .parse()
//     .unwrap();
//     info!("seed is {:?}", seed);
//     let config2 = generate_config(vec![seed]);
//
//     info!("start second worker");
//
//     let worker2 = NetworkWorker::new(Params::new(config2.clone(), protocol)).unwrap();
//     let service2 = worker2.service().clone();
//     let mut stream2 = service2.event_stream();
//
//     task::spawn(worker2);
//
//     thread::sleep(Duration::from_secs(1));
//
//     let data = vec![1, 2, 3, 4];
//     let data_clone = data.clone();
//     let addr1 = service1.peer_id().clone();
//
//     info!(
//         "first peer address is {:?} id is {:?},second peer address is {:?} id is {:?}",
//         config1.listen_addresses,
//         service1.local_peer_id(),
//         config2.listen_addresses,
//         service2.local_peer_id()
//     );
//
//     let fut = async move {
//         while let Some(event) = stream2.next().await {
//             match event {
//                 Event::NotificationStreamOpened { remote, info } => {
//                     info!("open stream from {},info is {:?}", remote, info);
//                     let result = service2.get_address(remote.clone()).await;
//                     info!("remote {} address is {:?}", remote, result);
//                     service2.write_notification(
//                         addr1.clone(),
//                         PROTOCOL_NAME.into(),
//                         data_clone.clone(),
//                     );
//                 }
//                 _ => {
//                     info!("event is {:?}", event);
//                 }
//             }
//         }
//     };
//
//     task::spawn(fut);
//
//     let fut = async move {
//         while let Some(event) = stream1.next().await {
//             match event {
//                 Event::NotificationsReceived {
//                     remote,
//                     protocol_name,
//                     mut messages,
//                 } => {
//                     let msg = messages.remove(0).to_vec();
//                     info!("receive message {:?} from {} ", msg, remote);
//                     assert_eq!(protocol_name.as_ref(), PROTOCOL_NAME);
//                     assert_eq!(msg, data);
//                     break;
//                 }
//                 Event::NotificationStreamOpened { remote, info } => {
//                     info!("open stream from {},info is {:?}", remote, info);
//                     let result = service1.get_address(remote.clone()).await;
//                     info!("remote {} address is {:?}", remote, result);
//                 }
//                 _ => {
//                     info!("event is {:?}", event);
//                 }
//             }
//         }
//     };
//
//     task::block_on(fut);
// }
//

#[stest::test]
async fn test_handshake_fail() {
    let protocol = ProtocolId::from("starcoin");
    let config1 = generate_config(vec![], vec![PROTOCOL_NAME.into()], vec![]);
    let status1 = Status::default();
    let worker1 = NetworkWorker::new(Params::new(
        config1.clone(),
        protocol.clone(),
        TestChainInfoHandle::new(status1),
        None,
    ))
    .unwrap();
    let service1 = worker1.service().clone();

    let _ = tokio::task::spawn(worker1);

    let seed = config::MultiaddrWithPeerId {
        multiaddr: config1.listen_addresses[0].clone(),
        peer_id: service1.local_peer_id(),
    };

    let config2 = generate_config(vec![seed], vec![PROTOCOL_NAME.into()], vec![]);
    let status2 = Status::default();

    let worker2 = NetworkWorker::new(Params::new(
        config2,
        protocol,
        TestChainInfoHandle::new(status2),
        None,
    ))
    .unwrap();
    let service2 = worker2.service().clone();

    let _ = tokio::task::spawn(worker2);
    tokio::time::sleep(Duration::from_secs(1)).await;

    debug!(
        "first peer is {:?},second peer is {:?}",
        service1.peer_id(),
        service2.peer_id()
    );
    let state1 = service1.network_state().await.unwrap();
    let state2 = service2.network_state().await.unwrap();

    assert_eq!(state1.connected_peers.len(), 0);
    assert_eq!(state2.connected_peers.len(), 0);
}

fn generate_config(
    boot_nodes: Vec<MultiaddrWithPeerId>,
    notif_protocols: Vec<Cow<'static, str>>,
    rpc_protocols: Vec<RequestResponseConfig>,
) -> NetworkConfiguration {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];

    config::NetworkConfiguration {
        //notifications_protocols: vec![(ENGINE_ID, From::from("/foo"))],
        notifications_protocols: notif_protocols,
        request_response_protocols: rpc_protocols,
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        boot_nodes,
        ..config::NetworkConfiguration::new_local()
    }
}

// test handshake message compatible with a serialize hex message.
#[test]
fn test_handshake_message() {
    // let mystr = r#"{"chain_id":{"id":1},"genesis_hash":"0x509224b8142926f6c079c66a85ca6db7981734bfe8f9427b3b925574be013f93","status":{"head":{"parent_hash":"0x82b85e25967cd4077f4df26a8975ab34ec6eba954e2c38d2b8393c6c42c2963c","timestamp":1612227819459,"number":9213,"author":"0xe6f6e9ec5a878e29350b4356e21d63db","author_auth_key":null,"txn_accumulator_root":"0xa57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f56","block_accumulator_root":"0x163305561261490852c28f3c1131e4e8d181bea0e1c8552f1ff9f8fbdd107727","state_root":"0xcead8e63f08b297df0e6c0e80a15f824d1a6f08ecb6f88021d6f3dc6c31544af","gas_used":16384000,"difficulty":"0x1648","body_hash":"0x19990c2875098a829ac4d6db2c78b77e6102d0837920304a14ebb474190a5007","chain_id":{"id":1},"nonce":620209232,"extra":"0x00000000"},"info":{"block_id":"0xcabe94c219acfae4044e8e5c8609a6d98153935e60e18be7f0ca611243714da2","total_difficulty":"0x0356fcbd","txn_accumulator_info":{"accumulator_root":"0xa57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f56","frozen_subtree_roots":["0xed2a8ca4a2972761099903410a9dc0c4607eaec944c41d919c27c57418d2aa59","0x21ee454f8510f89866eae45cd5727bee271595e67740ef5aaf80f9fc9d3b84d3","0x527890d7a348f2bfe9801eaad4d98facd340489a37234f405c15ab4e64a0f2eb","0xd0dacaa8beb77998983313ce06b44385b88c1772992f42a835b2f8477118321b","0x31b0df1da737424b169c3a43c0bc23794cc65d65d352aeff8a50b0593320a0cb","0x17dcc4f902c5e237a2c2a3b47b9263b7e67512c026ff76981e9c88955135cd86","0x0686841f7caeb4cd82eb1d51575971c7b189609a87c63970447c45b103619086","0xabfa4a9ed920176ad2a789d731f26398768732f813351e43a38d4c1aa22ff259","0x6914b1dd9aac5d4721fdb7bd736b1f107e72253050b4effd4bd9952da32eef84","0x2b0be3dc9f9196c5f8b5b9c430083d682720651154b29d1778971273eb9dfbcf","0x566f2db25b5255647988d164c4e2855b689fe5dcf7b1ba37bfa6a3d86accc503","0xe5b5f78b0b2e08fc3e3cafa9808346704da2f7b7a572dd84ed947e00003266c4"],"num_leaves":126960,"num_nodes":253908},"block_accumulator_info":{"accumulator_root":"0x2be16af3d9084b18d6ca44050ff46474d888b8c6340db0fbcb7aef9e423794af","frozen_subtree_roots":["0xef637a9b977e8969503e4fedb8558b0f294268bbaa6a0b24a824ad3c98edcf1e","0xa8cf073cfe1b08a5ed94a04dc79f16d125b7d4fb4d7ce02f75f412ded9cf9b79","0xf89ff07faba4299566955c4b9c31fcba99fc5855a229bed7d6487dafd59f1e70","0x2fd161c1b5d03833eb3efb09e530e689ac67ec7d5748246df4891bb9c3f3111b","0x55e40a53390e839a588904e16fe656676b0c5a7b3ec70bd8dcc2276e70e7600b","0xb3918be1fd6460dd30daf058e0e516c7046d242642130547f510335a319a98dd","0xf0737bc518a99c1a619bd87ba82d95dcd8dd19b0836a7dbed514b603f90e7ea8","0xf48e3dfc240d86a64e9adb9c2d276c6f42119e4aaee7598b13f61e4d77390d11","0x62cb92b81afa80226494d92a2120bdd4e9956c48f44f41b1283a59d9fe32e6df","0xeb5618d7d5699735477bee792b0e1a1ffa3c892fa31b7515b6948d80e3b424b2"],"num_leaves":9214,"num_nodes":18418}}}}"#;
    // let mychain_info: ChainInfo = serde_json::from_str::<ChainInfo>(mystr).unwrap();
    // let myencode = mychain_info.encode().unwrap();
    // println!("{myencode:?}");
    let json_msg = r#"
       {"version":1,"min_supported_version":1,
       "notif_protocols":["/starcoin/txn/1","/starcoin/block/1"],
       "rpc_protocols":[],
       "generic_data":[1, 32, 80, 146, 36, 184, 20, 41, 38, 246, 192, 121, 198, 106, 133, 202, 109, 183, 152, 23, 52, 191, 232, 249, 66, 123, 59, 146, 85, 116, 190, 1, 63, 147, 32, 130, 184, 94, 37, 150, 124, 212, 7, 127, 77, 242, 106, 137, 117, 171, 52, 236, 110, 186, 149, 78, 44, 56, 210, 184, 57, 60, 108, 66, 194, 150, 60, 195, 55, 68, 96, 119, 1, 0, 0, 253, 35, 0, 0, 0, 0, 0, 0, 230, 246, 233, 236, 90, 135, 142, 41, 53, 11, 67, 86, 226, 29, 99, 219, 0, 32, 165, 117, 22, 186, 80, 103, 42, 254, 35, 134, 149, 41, 178, 213, 75, 156, 185, 91, 246, 194, 173, 9, 130, 4, 140, 93, 193, 99, 62, 86, 127, 86, 32, 22, 51, 5, 86, 18, 97, 73, 8, 82, 194, 143, 60, 17, 49, 228, 232, 209, 129, 190, 160, 225, 200, 85, 47, 31, 249, 248, 251, 221, 16, 119, 39, 32, 206, 173, 142, 99, 240, 139, 41, 125, 240, 230, 192, 232, 10, 21, 248, 36, 209, 166, 240, 142, 203, 111, 136, 2, 29, 111, 61, 198, 195, 21, 68, 175, 0, 0, 250, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 72, 32, 25, 153, 12, 40, 117, 9, 138, 130, 154, 196, 214, 219, 44, 120, 183, 126, 97, 2, 208, 131, 121, 32, 48, 74, 20, 235, 180, 116, 25, 10, 80, 7, 1, 80, 164, 247, 36, 0, 0, 0, 0, 32, 202, 190, 148, 194, 25, 172, 250, 228, 4, 78, 142, 92, 134, 9, 166, 217, 129, 83, 147, 94, 96, 225, 139, 231, 240, 202, 97, 18, 67, 113, 77, 162, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 86, 252, 189, 32, 165, 117, 22, 186, 80, 103, 42, 254, 35, 134, 149, 41, 178, 213, 75, 156, 185, 91, 246, 194, 173, 9, 130, 4, 140, 93, 193, 99, 62, 86, 127, 86, 12, 32, 237, 42, 140, 164, 162, 151, 39, 97, 9, 153, 3, 65, 10, 157, 192, 196, 96, 126, 174, 201, 68, 196, 29, 145, 156, 39, 197, 116, 24, 210, 170, 89, 32, 33, 238, 69, 79, 133, 16, 248, 152, 102, 234, 228, 92, 213, 114, 123, 238, 39, 21, 149, 230, 119, 64, 239, 90, 175, 128, 249, 252, 157, 59, 132, 211, 32, 82, 120, 144, 215, 163, 72, 242, 191, 233, 128, 30, 170, 212, 217, 143, 172, 211, 64, 72, 154, 55, 35, 79, 64, 92, 21, 171, 78, 100, 160, 242, 235, 32, 208, 218, 202, 168, 190, 183, 121, 152, 152, 51, 19, 206, 6, 180, 67, 133, 184, 140, 23, 114, 153, 47, 66, 168, 53, 178, 248, 71, 113, 24, 50, 27, 32, 49, 176, 223, 29, 167, 55, 66, 75, 22, 156, 58, 67, 192, 188, 35, 121, 76, 198, 93, 101, 211, 82, 174, 255, 138, 80, 176, 89, 51, 32, 160, 203, 32, 23, 220, 196, 249, 2, 197, 226, 55, 162, 194, 163, 180, 123, 146, 99, 183, 230, 117, 18, 192, 38, 255, 118, 152, 30, 156, 136, 149, 81, 53, 205, 134, 32, 6, 134, 132, 31, 124, 174, 180, 205, 130, 235, 29, 81, 87, 89, 113, 199, 177, 137, 96, 154, 135, 198, 57, 112, 68, 124, 69, 177, 3, 97, 144, 134, 32, 171, 250, 74, 158, 217, 32, 23, 106, 210, 167, 137, 215, 49, 242, 99, 152, 118, 135, 50, 248, 19, 53, 30, 67, 163, 141, 76, 26, 162, 47, 242, 89, 32, 105, 20, 177, 221, 154, 172, 93, 71, 33, 253, 183, 189, 115, 107, 31, 16, 126, 114, 37, 48, 80, 180, 239, 253, 75, 217, 149, 45, 163, 46, 239, 132, 32, 43, 11, 227, 220, 159, 145, 150, 197, 248, 181, 185, 196, 48, 8, 61, 104, 39, 32, 101, 17, 84, 178, 157, 23, 120, 151, 18, 115, 235, 157, 251, 207, 32, 86, 111, 45, 178, 91, 82, 85, 100, 121, 136, 209, 100, 196, 226, 133, 91, 104, 159, 229, 220, 247, 177, 186, 55, 191, 166, 163, 216, 106, 204, 197, 3, 32, 229, 181, 247, 139, 11, 46, 8, 252, 62, 60, 175, 169, 128, 131, 70, 112, 77, 162, 247, 183, 165, 114, 221, 132, 237, 148, 126, 0, 0, 50, 102, 196, 240, 239, 1, 0, 0, 0, 0, 0, 212, 223, 3, 0, 0, 0, 0, 0, 32, 43, 225, 106, 243, 217, 8, 75, 24, 214, 202, 68, 5, 15, 244, 100, 116, 216, 136, 184, 198, 52, 13, 176, 251, 203, 122, 239, 158, 66, 55, 148, 175, 10, 32, 239, 99, 122, 155, 151, 126, 137, 105, 80, 62, 79, 237, 184, 85, 139, 15, 41, 66, 104, 187, 170, 106, 11, 36, 168, 36, 173, 60, 152, 237, 207, 30, 32, 168, 207, 7, 60, 254, 27, 8, 165, 237, 148, 160, 77, 199, 159, 22, 209, 37, 183, 212, 251, 77, 124, 224, 47, 117, 244, 18, 222, 217, 207, 155, 121, 32, 248, 159, 240, 127, 171, 164, 41, 149, 102, 149, 92, 75, 156, 49, 252, 186, 153, 252, 88, 85, 162, 41, 190, 215, 214, 72, 125, 175, 213, 159, 30, 112, 32, 47, 209, 97, 193, 181, 208, 56, 51, 235, 62, 251, 9, 229, 48, 230, 137, 172, 103, 236, 125, 87, 72, 36, 109, 244, 137, 27, 185, 195, 243, 17, 27, 32, 85, 228, 10, 83, 57, 14, 131, 154, 88, 137, 4, 225, 111, 230, 86, 103, 107, 12, 90, 123, 62, 199, 11, 216, 220, 194, 39, 110, 112, 231, 96, 11, 32, 179, 145, 139, 225, 253, 100, 96, 221, 48, 218, 240, 88, 224, 229, 22, 199, 4, 109, 36, 38, 66, 19, 5, 71, 245, 16, 51, 90, 49, 154, 152, 221, 32, 240, 115, 123, 197, 24, 169, 156, 26, 97, 155, 216, 123, 168, 45, 149, 220, 216, 221, 25, 176, 131, 106, 125, 190, 213, 20, 182, 3, 249, 14, 126, 168, 32, 244, 142, 61, 252, 36, 13, 134, 166, 78, 154, 219, 156, 45, 39, 108, 111, 66, 17, 158, 74, 174, 231, 89, 139, 19, 246, 30, 77, 119, 57, 13, 17, 32, 98, 203, 146, 184, 26, 250, 128, 34, 100, 148, 217, 42, 33, 32, 189, 212, 233, 149, 108, 72, 244, 79, 65, 177, 40, 58, 89, 217, 254, 50, 230, 223, 32, 235, 86, 24, 215, 213, 105, 151, 53, 71, 123, 238, 121, 43, 14, 26, 31, 250, 60, 137, 47, 163, 27, 117, 21, 182, 148, 141, 128, 227, 180, 36, 178, 254, 35, 0, 0, 0, 0, 0, 0, 242, 71, 0, 0, 0, 0, 0, 0]}
       "#;
    let status = serde_json::from_str::<Status>(json_msg).unwrap();
    // let hex = hex::encode(status.encode().unwrap());
    // println!("{}", hex);
    // println!("{}", serde_json::to_string(&status).unwrap());
    let bin_msg = "0100000001000000020f2f73746172636f696e2f74786e2f31112f73746172636f696e2f626c6f636b2f310094090120509224b8142926f6c079c66a85ca6db7981734bfe8f9427b3b925574be013f932082b85e25967cd4077f4df26a8975ab34ec6eba954e2c38d2b8393c6c42c2963cc337446077010000fd23000000000000e6f6e9ec5a878e29350b4356e21d63db0020a57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f5620163305561261490852c28f3c1131e4e8d181bea0e1c8552f1ff9f8fbdd10772720cead8e63f08b297df0e6c0e80a15f824d1a6f08ecb6f88021d6f3dc6c31544af0000fa000000000000000000000000000000000000000000000000000000000000000000000016482019990c2875098a829ac4d6db2c78b77e6102d0837920304a14ebb474190a50070150a4f7240000000020cabe94c219acfae4044e8e5c8609a6d98153935e60e18be7f0ca611243714da2000000000000000000000000000000000000000000000000000000000356fcbd20a57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f560c20ed2a8ca4a2972761099903410a9dc0c4607eaec944c41d919c27c57418d2aa592021ee454f8510f89866eae45cd5727bee271595e67740ef5aaf80f9fc9d3b84d320527890d7a348f2bfe9801eaad4d98facd340489a37234f405c15ab4e64a0f2eb20d0dacaa8beb77998983313ce06b44385b88c1772992f42a835b2f8477118321b2031b0df1da737424b169c3a43c0bc23794cc65d65d352aeff8a50b0593320a0cb2017dcc4f902c5e237a2c2a3b47b9263b7e67512c026ff76981e9c88955135cd86200686841f7caeb4cd82eb1d51575971c7b189609a87c63970447c45b10361908620abfa4a9ed920176ad2a789d731f26398768732f813351e43a38d4c1aa22ff259206914b1dd9aac5d4721fdb7bd736b1f107e72253050b4effd4bd9952da32eef84202b0be3dc9f9196c5f8b5b9c430083d682720651154b29d1778971273eb9dfbcf20566f2db25b5255647988d164c4e2855b689fe5dcf7b1ba37bfa6a3d86accc50320e5b5f78b0b2e08fc3e3cafa9808346704da2f7b7a572dd84ed947e00003266c4f0ef010000000000d4df030000000000202be16af3d9084b18d6ca44050ff46474d888b8c6340db0fbcb7aef9e423794af0a20ef637a9b977e8969503e4fedb8558b0f294268bbaa6a0b24a824ad3c98edcf1e20a8cf073cfe1b08a5ed94a04dc79f16d125b7d4fb4d7ce02f75f412ded9cf9b7920f89ff07faba4299566955c4b9c31fcba99fc5855a229bed7d6487dafd59f1e70202fd161c1b5d03833eb3efb09e530e689ac67ec7d5748246df4891bb9c3f3111b2055e40a53390e839a588904e16fe656676b0c5a7b3ec70bd8dcc2276e70e7600b20b3918be1fd6460dd30daf058e0e516c7046d242642130547f510335a319a98dd20f0737bc518a99c1a619bd87ba82d95dcd8dd19b0836a7dbed514b603f90e7ea820f48e3dfc240d86a64e9adb9c2d276c6f42119e4aaee7598b13f61e4d77390d112062cb92b81afa80226494d92a2120bdd4e9956c48f44f41b1283a59d9fe32e6df20eb5618d7d5699735477bee792b0e1a1ffa3c892fa31b7515b6948d80e3b424b2fe23000000000000f247000000000000";
    let bytes = hex::decode(bin_msg).unwrap();
    let status2 = Status::decode(bytes.as_slice()).unwrap();
    assert_eq!(status, status2);
}

#[stest::test]
async fn test_support_protocol() {
    let protocol = ProtocolId::from("starcoin");
    let txn_v1 = "/starcoin/txn/1";
    let block_v1 = "/starcoin/block/1";
    let get_block_rpc = RequestResponseConfig {
        name: "/starcoin/rpc/get_blocks".into(),
        max_request_size: 1024,
        max_response_size: 1024,
        request_timeout: Duration::from_millis(1000),
        inbound_queue: None,
    };
    let config1 = generate_config(
        vec![],
        vec![block_v1.into(), txn_v1.into()],
        vec![get_block_rpc],
    );
    let status1 = Status::default();
    let worker1 = NetworkWorker::new(Params::new(
        config1.clone(),
        protocol.clone(),
        TestChainInfoHandle::new(status1.clone()),
        None,
    ))
    .unwrap();
    let service1 = worker1.service().clone();
    let stream1 = service1.event_stream("test1");
    let _ = tokio::task::spawn(worker1);

    let seed = MultiaddrWithPeerId {
        multiaddr: config1.listen_addresses[0].clone(),
        peer_id: service1.local_peer_id(),
    };

    let config2 = generate_config(vec![seed], vec![block_v1.into()], vec![]);

    let worker2 = NetworkWorker::new(Params::new(
        config2,
        protocol,
        TestChainInfoHandle::new(status1),
        None,
    ))
    .unwrap();
    let service2 = worker2.service().clone();
    let stream2 = service2.event_stream("test1");
    let _ = tokio::task::spawn(worker2);

    tokio::time::sleep(Duration::from_secs(1)).await;

    debug!(
        "first peer is {:?},second peer is {:?}",
        service1.peer_id(),
        service2.peer_id()
    );
    let state1 = service1.network_state().await.unwrap();
    let state2 = service2.network_state().await.unwrap();
    assert_eq!(state1.connected_peers.len(), 1);
    assert_eq!(state2.connected_peers.len(), 1);

    let open_event1 = stream1
        .filter(|event| future::ready(matches!(event, Event::NotificationStreamOpened { .. })))
        .take(1)
        .collect::<Vec<_>>()
        .await
        .pop()
        .unwrap();

    if let NotificationStreamOpened {
        remote,
        protocol: _,
        generic_data: _,
        notif_protocols,
        rpc_protocols,
        version_string: _,
    } = open_event1
    {
        assert_eq!(&remote, service2.peer_id());
        assert_eq!(notif_protocols.len(), 1);
        assert_eq!(rpc_protocols.len(), 0);
    } else {
        panic!("Unexpected event type: {:?}", open_event1)
    }

    let open_event2 = stream2
        .filter(|event| future::ready(matches!(event, Event::NotificationStreamOpened { .. })))
        .take(1)
        .collect::<Vec<_>>()
        .await
        .pop()
        .unwrap();
    if let Event::NotificationStreamOpened {
        remote,
        protocol: _,
        generic_data: _,
        notif_protocols,
        rpc_protocols,
        version_string: _,
    } = open_event2
    {
        assert_eq!(&remote, service1.peer_id());
        assert_eq!(notif_protocols.len(), 2);
        assert_eq!(rpc_protocols.len(), 1);
    } else {
        panic!("Unexpected event type: {:?}", open_event2)
    }
}
