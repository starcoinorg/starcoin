// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service::NetworkStateInfo;
use crate::{config, Event, NetworkService, NetworkWorker};
use crate::{NetworkConfiguration, Params, ProtocolId};
use async_std::task;
use futures::executor::block_on;
use futures::prelude::*;
use futures::stream::StreamExt;
use libp2p::PeerId;
use network_p2p_types::MultiaddrWithPeerId;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::thread;
use std::{sync::Arc, time::Duration};

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
        reserved_nodes: vec![config::MultiaddrWithPeerId {
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
            reserved_nodes: vec![config::MultiaddrWithPeerId {
                multiaddr: listen_addr.clone(),
                peer_id: main_node_peer_id,
            }],
            transport: config::TransportConfig::MemoryOnly,
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
                Event::NotificationsReceived {
                    messages,
                    protocol: protocol_name,
                    ..
                } => {
                    for message in messages {
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
                Event::NotificationStreamOpened { .. } => break,
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
    let reserved_node = config::MultiaddrWithPeerId {
        multiaddr: config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)],
        peer_id: PeerId::random(),
    };

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        reserved_nodes: vec![reserved_node],
        ..config::NetworkConfiguration::new("test-node", "test-client", Default::default())
    });
}

#[test]
#[should_panic(expected = "don't match the transport")]
fn ensure_reserved_node_addresses_consistent_with_transport_not_memory() {
    let listen_addr = config::build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(0_u16)];
    let reserved_node = config::MultiaddrWithPeerId {
        multiaddr: config::build_multiaddr![Memory(rand::random::<u64>())],
        peer_id: PeerId::random(),
    };

    let _ = build_test_full_node(config::NetworkConfiguration {
        listen_addresses: vec![listen_addr],
        reserved_nodes: vec![reserved_node],
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
    let config1 = generate_config(vec![]);
    let chain1 = ChainInfo::random();
    let worker1 =
        NetworkWorker::new(Params::new(config1.clone(), protocol.clone(), chain1, None)).unwrap();
    let service1 = worker1.service().clone();

    task::spawn(worker1);

    let seed = config::MultiaddrWithPeerId {
        multiaddr: config1.listen_addresses[0].clone(),
        peer_id: service1.local_peer_id(),
    };

    let config2 = generate_config(vec![seed]);
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

fn generate_config(boot_nodes: Vec<MultiaddrWithPeerId>) -> NetworkConfiguration {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];

    config::NetworkConfiguration {
        //notifications_protocols: vec![(ENGINE_ID, From::from("/foo"))],
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![listen_addr],
        transport: config::TransportConfig::MemoryOnly,
        boot_nodes,
        ..config::NetworkConfiguration::new_local()
    }
}
