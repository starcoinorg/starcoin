// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::RequestResponseConfig;
use crate::protocol::message::generic::Status;
use crate::service::NetworkStateInfo;
use crate::{config, Event, NetworkService, NetworkWorker};
use crate::{NetworkConfiguration, Params, ProtocolId};
use async_std::task;
use bcs_ext::BCSCodec;
use futures::executor::block_on;
use futures::prelude::*;
use futures::stream::StreamExt;
use libp2p::PeerId;
use network_p2p_types::MultiaddrWithPeerId;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::borrow::Cow;
use std::thread;
use std::{sync::Arc, time::Duration};
use Event::NotificationStreamOpened;

static TEST_CHAIN_INFO: Lazy<ChainInfo> =
    Lazy::new(|| ChainInfo::new(ChainId::new(0), HashValue::zero(), ChainStatus::random()));

/// Builds a full node to be used for testing. Returns the node service and its associated events
/// stream.
///
/// > **Note**: We return the events stream in order to not possibly lose events between the
/// >   construction of the service and the moment the events stream is grabbed.
fn build_test_full_node(
    config: config::NetworkConfiguration,
) -> (Arc<NetworkService>, impl Stream<Item = Event>) {
    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from("/test-protocol-name"),
        chain_info: TEST_CHAIN_INFO.clone(),
        metrics_registry: None,
    })
    .unwrap();

    let service = worker.service().clone();
    let event_stream = service.event_stream("test");

    async_std::task::spawn(async move {
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
fn lots_of_incoming_peers_works() {
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

    futures::executor::block_on(async move { future::join_all(background_tasks_to_wait).await });
}

#[stest::test(timeout = 600)]
fn notifications_back_pressure() {
    // Node 1 floods node 2 with notifications. Random sleeps are done on node 2 to simulate the
    // node being busy. We make sure that all notifications are received.

    const TOTAL_NOTIFS: usize = 10_000;

    let (node1, mut events_stream1, node2, mut events_stream2) = build_nodes_one_proto();
    let node2_id = node2.local_peer_id();

    let receiver = async_std::task::spawn(async move {
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
                async_std::task::sleep(Duration::from_millis(rand::random::<u64>() % 750)).await;
            }
        }
    });
    async_std::task::block_on(async move {
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

        receiver.await;
    });
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
fn test_handshake_fail() {
    let protocol = ProtocolId::from("starcoin");
    let config1 = generate_config(vec![], vec![PROTOCOL_NAME.into()], vec![]);
    let chain1 = ChainInfo::random();
    let worker1 =
        NetworkWorker::new(Params::new(config1.clone(), protocol.clone(), chain1, None)).unwrap();
    let service1 = worker1.service().clone();

    task::spawn(worker1);

    let seed = config::MultiaddrWithPeerId {
        multiaddr: config1.listen_addresses[0].clone(),
        peer_id: service1.local_peer_id(),
    };

    let config2 = generate_config(vec![seed], vec![PROTOCOL_NAME.into()], vec![]);
    let chain2 = ChainInfo::random();

    let worker2 = NetworkWorker::new(Params::new(config2, protocol, chain2, None)).unwrap();
    let service2 = worker2.service().clone();

    task::spawn(worker2);

    thread::sleep(Duration::from_secs(1));

    debug!(
        "first peer is {:?},second peer is {:?}",
        service1.peer_id(),
        service2.peer_id()
    );
    let state1 = block_on(async { service1.network_state().await }).unwrap();
    let state2 = block_on(async { service2.network_state().await }).unwrap();

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
    let json_msg = r#"
       {"version":1,"min_supported_version":1,
       "notif_protocols":["/starcoin/txn/1","/starcoin/block/1"],
       "rpc_protocols":[],
       "info":{"chain_id":{"id":1},"genesis_hash":"0x509224b8142926f6c079c66a85ca6db7981734bfe8f9427b3b925574be013f93","status":{"head":{"parent_hash":"0x82b85e25967cd4077f4df26a8975ab34ec6eba954e2c38d2b8393c6c42c2963c","timestamp":1612227819459,"number":9213,"author":"0xe6f6e9ec5a878e29350b4356e21d63db","author_auth_key":null,"txn_accumulator_root":"0xa57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f56","block_accumulator_root":"0x163305561261490852c28f3c1131e4e8d181bea0e1c8552f1ff9f8fbdd107727","state_root":"0xcead8e63f08b297df0e6c0e80a15f824d1a6f08ecb6f88021d6f3dc6c31544af","gas_used":16384000,"difficulty":"0x1648","body_hash":"0x19990c2875098a829ac4d6db2c78b77e6102d0837920304a14ebb474190a5007","chain_id":{"id":1},"nonce":620209232,"extra":"0x00000000"},"info":{"block_id":"0xcabe94c219acfae4044e8e5c8609a6d98153935e60e18be7f0ca611243714da2","total_difficulty":"0x0356fcbd","txn_accumulator_info":{"accumulator_root":"0xa57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f56","frozen_subtree_roots":["0xed2a8ca4a2972761099903410a9dc0c4607eaec944c41d919c27c57418d2aa59","0x21ee454f8510f89866eae45cd5727bee271595e67740ef5aaf80f9fc9d3b84d3","0x527890d7a348f2bfe9801eaad4d98facd340489a37234f405c15ab4e64a0f2eb","0xd0dacaa8beb77998983313ce06b44385b88c1772992f42a835b2f8477118321b","0x31b0df1da737424b169c3a43c0bc23794cc65d65d352aeff8a50b0593320a0cb","0x17dcc4f902c5e237a2c2a3b47b9263b7e67512c026ff76981e9c88955135cd86","0x0686841f7caeb4cd82eb1d51575971c7b189609a87c63970447c45b103619086","0xabfa4a9ed920176ad2a789d731f26398768732f813351e43a38d4c1aa22ff259","0x6914b1dd9aac5d4721fdb7bd736b1f107e72253050b4effd4bd9952da32eef84","0x2b0be3dc9f9196c5f8b5b9c430083d682720651154b29d1778971273eb9dfbcf","0x566f2db25b5255647988d164c4e2855b689fe5dcf7b1ba37bfa6a3d86accc503","0xe5b5f78b0b2e08fc3e3cafa9808346704da2f7b7a572dd84ed947e00003266c4"],"num_leaves":126960,"num_nodes":253908},"block_accumulator_info":{"accumulator_root":"0x2be16af3d9084b18d6ca44050ff46474d888b8c6340db0fbcb7aef9e423794af","frozen_subtree_roots":["0xef637a9b977e8969503e4fedb8558b0f294268bbaa6a0b24a824ad3c98edcf1e","0xa8cf073cfe1b08a5ed94a04dc79f16d125b7d4fb4d7ce02f75f412ded9cf9b79","0xf89ff07faba4299566955c4b9c31fcba99fc5855a229bed7d6487dafd59f1e70","0x2fd161c1b5d03833eb3efb09e530e689ac67ec7d5748246df4891bb9c3f3111b","0x55e40a53390e839a588904e16fe656676b0c5a7b3ec70bd8dcc2276e70e7600b","0xb3918be1fd6460dd30daf058e0e516c7046d242642130547f510335a319a98dd","0xf0737bc518a99c1a619bd87ba82d95dcd8dd19b0836a7dbed514b603f90e7ea8","0xf48e3dfc240d86a64e9adb9c2d276c6f42119e4aaee7598b13f61e4d77390d11","0x62cb92b81afa80226494d92a2120bdd4e9956c48f44f41b1283a59d9fe32e6df","0xeb5618d7d5699735477bee792b0e1a1ffa3c892fa31b7515b6948d80e3b424b2"],"num_leaves":9214,"num_nodes":18418}}}}}
       "#;
    let status = serde_json::from_str::<Status>(json_msg).unwrap();
    //let hex = hex::encode(status.encode().unwrap());
    //println!("{}", hex);
    //println!("{}", serde_json::to_string(&status).unwrap());
    let bin_msg = "0100000001000000020f2f73746172636f696e2f74786e2f31112f73746172636f696e2f626c6f636b2f31000120509224b8142926f6c079c66a85ca6db7981734bfe8f9427b3b925574be013f932082b85e25967cd4077f4df26a8975ab34ec6eba954e2c38d2b8393c6c42c2963cc337446077010000fd23000000000000e6f6e9ec5a878e29350b4356e21d63db0020a57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f5620163305561261490852c28f3c1131e4e8d181bea0e1c8552f1ff9f8fbdd10772720cead8e63f08b297df0e6c0e80a15f824d1a6f08ecb6f88021d6f3dc6c31544af0000fa000000000000000000000000000000000000000000000000000000000000000000000016482019990c2875098a829ac4d6db2c78b77e6102d0837920304a14ebb474190a50070150a4f7240000000020cabe94c219acfae4044e8e5c8609a6d98153935e60e18be7f0ca611243714da2000000000000000000000000000000000000000000000000000000000356fcbd20a57516ba50672afe23869529b2d54b9cb95bf6c2ad0982048c5dc1633e567f560c20ed2a8ca4a2972761099903410a9dc0c4607eaec944c41d919c27c57418d2aa592021ee454f8510f89866eae45cd5727bee271595e67740ef5aaf80f9fc9d3b84d320527890d7a348f2bfe9801eaad4d98facd340489a37234f405c15ab4e64a0f2eb20d0dacaa8beb77998983313ce06b44385b88c1772992f42a835b2f8477118321b2031b0df1da737424b169c3a43c0bc23794cc65d65d352aeff8a50b0593320a0cb2017dcc4f902c5e237a2c2a3b47b9263b7e67512c026ff76981e9c88955135cd86200686841f7caeb4cd82eb1d51575971c7b189609a87c63970447c45b10361908620abfa4a9ed920176ad2a789d731f26398768732f813351e43a38d4c1aa22ff259206914b1dd9aac5d4721fdb7bd736b1f107e72253050b4effd4bd9952da32eef84202b0be3dc9f9196c5f8b5b9c430083d682720651154b29d1778971273eb9dfbcf20566f2db25b5255647988d164c4e2855b689fe5dcf7b1ba37bfa6a3d86accc50320e5b5f78b0b2e08fc3e3cafa9808346704da2f7b7a572dd84ed947e00003266c4f0ef010000000000d4df030000000000202be16af3d9084b18d6ca44050ff46474d888b8c6340db0fbcb7aef9e423794af0a20ef637a9b977e8969503e4fedb8558b0f294268bbaa6a0b24a824ad3c98edcf1e20a8cf073cfe1b08a5ed94a04dc79f16d125b7d4fb4d7ce02f75f412ded9cf9b7920f89ff07faba4299566955c4b9c31fcba99fc5855a229bed7d6487dafd59f1e70202fd161c1b5d03833eb3efb09e530e689ac67ec7d5748246df4891bb9c3f3111b2055e40a53390e839a588904e16fe656676b0c5a7b3ec70bd8dcc2276e70e7600b20b3918be1fd6460dd30daf058e0e516c7046d242642130547f510335a319a98dd20f0737bc518a99c1a619bd87ba82d95dcd8dd19b0836a7dbed514b603f90e7ea820f48e3dfc240d86a64e9adb9c2d276c6f42119e4aaee7598b13f61e4d77390d112062cb92b81afa80226494d92a2120bdd4e9956c48f44f41b1283a59d9fe32e6df20eb5618d7d5699735477bee792b0e1a1ffa3c892fa31b7515b6948d80e3b424b2fe23000000000000f247000000000000";
    let bytes = hex::decode(bin_msg).unwrap();
    let status2 = Status::decode(bytes.as_slice()).unwrap();
    assert_eq!(status, status2);
}

#[stest::test]
fn test_support_protocol() {
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
    let chain1 = ChainInfo::random();
    let worker1 = NetworkWorker::new(Params::new(
        config1.clone(),
        protocol.clone(),
        chain1.clone(),
        None,
    ))
    .unwrap();
    let service1 = worker1.service().clone();
    let stream1 = service1.event_stream("test1");
    task::spawn(worker1);

    let seed = config::MultiaddrWithPeerId {
        multiaddr: config1.listen_addresses[0].clone(),
        peer_id: service1.local_peer_id(),
    };

    let config2 = generate_config(vec![seed], vec![block_v1.into()], vec![]);

    let worker2 = NetworkWorker::new(Params::new(config2, protocol, chain1, None)).unwrap();
    let service2 = worker2.service().clone();
    let stream2 = service2.event_stream("test1");
    task::spawn(worker2);

    thread::sleep(Duration::from_secs(1));

    debug!(
        "first peer is {:?},second peer is {:?}",
        service1.peer_id(),
        service2.peer_id()
    );
    let state1 = block_on(async { service1.network_state().await }).unwrap();
    let state2 = block_on(async { service2.network_state().await }).unwrap();

    assert_eq!(state1.connected_peers.len(), 1);
    assert_eq!(state2.connected_peers.len(), 1);

    let open_event1 = block_on(async {
        stream1
            .filter(|event| future::ready(matches!(event, Event::NotificationStreamOpened { .. })))
            .take(1)
            .collect::<Vec<_>>()
            .await
    })
    .pop()
    .unwrap();
    if let Event::NotificationStreamOpened {
        remote,
        protocol: _,
        info: _,
        notif_protocols,
        rpc_protocols,
    } = open_event1
    {
        assert_eq!(&remote, service2.peer_id());
        assert_eq!(notif_protocols.len(), 1);
        assert_eq!(rpc_protocols.len(), 0);
    } else {
        panic!("Unexpected event type: {:?}", open_event1)
    }

    let open_event2 = block_on(async {
        stream2
            .filter(|event| future::ready(matches!(event, Event::NotificationStreamOpened { .. })))
            .take(1)
            .collect::<Vec<_>>()
            .await
    })
    .pop()
    .unwrap();
    if let Event::NotificationStreamOpened {
        remote,
        protocol: _,
        info: _,
        notif_protocols,
        rpc_protocols,
    } = open_event2
    {
        assert_eq!(&remote, service1.peer_id());
        assert_eq!(notif_protocols.len(), 2);
        assert_eq!(rpc_protocols.len(), 1);
    } else {
        panic!("Unexpected event type: {:?}", open_event2)
    }
}
