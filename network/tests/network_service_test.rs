// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use async_std::task;
use futures::stream::StreamExt;
use futures_timer::Delay;
use logger::prelude::*;
use network_api::messages::{NotificationMessage, PeerMessage, TransactionsMessage};
use network_api::{Multiaddr, NetworkService};
use network_p2p_types::{random_memory_addr, MultiaddrWithPeerId};
use starcoin_config::{BuiltinNetworkID, NetworkConfig, NodeConfig};
use starcoin_crypto::hash::HashValue;
use starcoin_network::build_network_worker;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::SignedUserTransaction;
use std::sync::Arc;
use std::{thread, time::Duration};

pub type NetworkComponent = (Arc<network_p2p::NetworkService>, NetworkConfig);

fn build_test_network_pair() -> (NetworkComponent, NetworkComponent) {
    let mut l = build_test_network_services(2).into_iter();
    let a = l.next().unwrap();
    let b = l.next().unwrap();
    (a, b)
}

fn build_test_network_services(num: usize) -> Vec<NetworkComponent> {
    let mut result: Vec<NetworkComponent> = Vec::with_capacity(num);
    let mut first_addr: Option<Multiaddr> = None;
    let chain_info = ChainInfo::new(
        BuiltinNetworkID::Test.chain_id(),
        HashValue::random(),
        ChainStatus::random(),
    );
    for _index in 0..num {
        let mut boot_nodes = Vec::new();

        if let Some(first_addr) = first_addr.as_ref() {
            boot_nodes.push(MultiaddrWithPeerId::new(
                first_addr.clone(),
                result[0].0.peer_id().clone(),
            ));
        }
        let mut node_config = NodeConfig::random_for_test();

        node_config.network.listen = random_memory_addr();
        node_config.network.seeds = boot_nodes;

        info!(
            "listen:{:?},boots {:?}",
            node_config.network.listen, node_config.network.seeds
        );
        if first_addr.is_none() {
            first_addr = Some(node_config.network.listen.clone());
        }
        let mut protocols = NotificationMessage::protocols();
        protocols.push(TEST_NOTIF_PROTOCOL_NAME.into());
        let worker =
            build_network_worker(&node_config, chain_info.clone(), protocols, None).unwrap();
        let network_service = worker.service().clone();
        async_std::task::spawn(worker);
        result.push({
            let c: NetworkComponent = (network_service, node_config.network.clone());
            c
        });
    }
    result
}

const TEST_NOTIF_PROTOCOL_NAME: &str = "/test_notif";
#[test]
fn test_send_receive() {
    ::logger::init_for_test();
    //let mut rt = Builder::new().core_threads(1).build().unwrap();
    let ((service1, _), (service2, _)) = build_test_network_pair();
    let msg_peer_id_1 = service1.peer_id().clone();
    let msg_peer_id_2 = service2.peer_id().clone();
    let receiver_1 = service1.event_stream("test");
    let receiver_2 = service2.event_stream("test");
    let total_message = 1000;
    thread::sleep(Duration::from_secs(1));
    let sender_fut = async move {
        for i in 0..total_message {
            debug!("message index is {}", i);
            Delay::new(Duration::from_millis(1)).await;
            let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();

            if i % 2 == 0 {
                service2.write_notification(
                    msg_peer_id_1.clone(),
                    std::borrow::Cow::Borrowed(TEST_NOTIF_PROTOCOL_NAME),
                    random_bytes,
                );
            } else {
                service1.write_notification(
                    msg_peer_id_2.clone(),
                    std::borrow::Cow::Borrowed(TEST_NOTIF_PROTOCOL_NAME),
                    random_bytes,
                );
            }
        }
    };

    let peer1_receiver_handle = task::spawn(receiver_1.take(total_message / 2).collect::<Vec<_>>());
    let peer2_receiver_handle = task::spawn(receiver_2.take(total_message / 2).collect::<Vec<_>>());
    task::spawn(sender_fut);

    let task = async move {
        let peer1_received_events = peer1_receiver_handle.await;
        let peer2_received_events = peer2_receiver_handle.await;
        assert_eq!(total_message / 2, peer1_received_events.len());
        assert_eq!(total_message / 2, peer2_received_events.len());
    };
    task::block_on(async_std::future::timeout(Duration::from_secs(10), task)).unwrap();
}

#[test]
fn test_connected_nodes() {
    ::logger::init_for_test();

    let (service1, service2) = build_test_network_pair();
    thread::sleep(Duration::from_secs(2));
    let fut = async move {
        assert_eq!(
            service1.0.is_connected(service2.0.peer_id().clone()).await,
            true
        );
    };
    task::block_on(fut);
}

// #[stest::test]
// async fn test_event_dht() {
//     let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();
//     let event = Event::Dht(DhtEvent::ValuePut(random_bytes.clone().into()));
//     test_handle_event(event).await;
// }
//
// #[stest::test]
// async fn test_event_notify_open() {
//     let event = Event::NotificationStreamOpened {
//         remote: PeerId::random(),
//         info: Box::new(ChainInfo::random()),
//     };
//     test_handle_event(event).await;
// }
//
// #[stest::test]
// async fn test_event_notify_close() {
//     let event = Event::NotificationStreamClosed {
//         remote: PeerId::random(),
//     };
//     test_handle_event(event).await;
// }

#[stest::test]
async fn test_event_notify_receive() {
    let (network1, network2) = test_helper::build_network_pair().await.unwrap();
    let msg = PeerMessage::new_transactions(
        network2.peer_id(),
        TransactionsMessage::new(vec![SignedUserTransaction::mock()]),
    );
    let mut receiver = network2.message_handler.channel();
    network1.service_ref.send_peer_message(msg.clone());
    let msg2 = receiver.next().await.unwrap();
    assert_eq!(msg.notification, msg2.notification);
}
