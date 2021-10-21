// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use async_std::task;
use futures::stream::StreamExt;
use futures_timer::Delay;
use network_api::messages::{
    Announcement, AnnouncementType, CompactBlockMessage, NotificationMessage, PeerMessage,
    TransactionsMessage, ANNOUNCEMENT_PROTOCOL_NAME, TXN_PROTOCOL_NAME,
};
use network_api::{Multiaddr, NetworkService};
use network_p2p_types::MultiaddrWithPeerId;
use starcoin_config::{BuiltinNetworkID, NetworkConfig, NodeConfig};
use starcoin_crypto::hash::HashValue;
use starcoin_logger::prelude::*;
use starcoin_network::build_network_worker;
use starcoin_types::block::{AccumulatorInfo, Block, BlockBody, BlockHeader, BlockInfo};
use starcoin_types::cmpact_block::CompactBlock;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::U256;
use std::sync::Arc;
use std::{thread, time::Duration};
use test_helper::network::build_network_with_config;

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
                *result[0].0.peer_id(),
            ));
        }
        let mut node_config = NodeConfig::random_for_test();

        node_config.network.seeds = boot_nodes.into();

        info!(
            "listen:{:?},boots {:?}",
            node_config.network.listen(),
            node_config.network.seeds
        );
        if first_addr.is_none() {
            first_addr = Some(node_config.network.listen());
        }
        let mut protocols = NotificationMessage::protocols();
        protocols.push(TEST_NOTIF_PROTOCOL_NAME.into());
        let (_peer_info, worker) = build_network_worker(
            &node_config.network,
            chain_info.clone(),
            protocols,
            None,
            None,
        )
        .unwrap();
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

#[stest::test]
fn test_send_receive() {
    let ((service1, _), (service2, _)) = build_test_network_pair();
    let msg_peer_id_1 = *service1.peer_id();
    let msg_peer_id_2 = *service2.peer_id();
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
                    msg_peer_id_1,
                    std::borrow::Cow::Borrowed(TEST_NOTIF_PROTOCOL_NAME),
                    random_bytes,
                );
            } else {
                service1.write_notification(
                    msg_peer_id_2,
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

#[stest::test]
fn test_connected_nodes() {
    let (service1, service2) = build_test_network_pair();
    thread::sleep(Duration::from_secs(2));
    let fut = async move {
        assert!(service1.0.is_connected(*service2.0.peer_id()).await);
        assert!(service2.0.is_connected(*service1.0.peer_id()).await);
    };
    task::block_on(fut);
}

#[stest::test]
async fn test_event_notify_receive() {
    let (network1, network2) = test_helper::build_network_pair().await.unwrap();
    // transaction
    let msg_send = PeerMessage::new_transactions(
        network2.peer_id(),
        TransactionsMessage::new(vec![SignedUserTransaction::mock()]),
    );
    let mut receiver = network2.message_handler.channel();
    network1.service_ref.send_peer_message(msg_send.clone());
    let msg_receive = receiver.next().await.unwrap();
    assert_eq!(msg_send.notification, msg_receive.notification);

    //block
    let msg_send = PeerMessage::new_compact_block(
        network2.peer_id(),
        CompactBlockMessage::new(
            CompactBlock::new(Block::new(BlockHeader::random(), BlockBody::new_empty())),
            mock_block_info(1.into()),
        ),
    );
    let mut receiver = network2.message_handler.channel();
    network1.service_ref.send_peer_message(msg_send.clone());
    let msg_receive = receiver.next().await.unwrap();
    assert_eq!(msg_send.notification, msg_receive.notification);
}

#[stest::test]
async fn test_event_notify_receive_repeat_block() {
    let (network1, network2) = test_helper::build_network_pair().await.unwrap();

    let block = Block::new(BlockHeader::random(), BlockBody::new_empty());

    let msg_send1 = PeerMessage::new_compact_block(
        network2.peer_id(),
        CompactBlockMessage::new(CompactBlock::new(block.clone()), mock_block_info(1.into())),
    );

    let msg_send2 = PeerMessage::new_compact_block(
        network2.peer_id(),
        CompactBlockMessage::new(CompactBlock::new(block.clone()), mock_block_info(1.into())),
    );

    let mut receiver = network2.message_handler.channel();
    network1.service_ref.send_peer_message(msg_send1.clone());
    network1.service_ref.send_peer_message(msg_send2.clone());
    let msg_receive1 = receiver.next().await.unwrap();
    assert_eq!(msg_send1.notification, msg_receive1.notification);

    //repeat message is filter, so expect timeout error.
    let msg_receive2 = async_std::future::timeout(Duration::from_secs(2), receiver.next()).await;
    assert!(msg_receive2.is_err());
}

#[stest::test]
async fn test_event_notify_receive_repeat_transaction() {
    let (network1, network2) = test_helper::build_network_pair().await.unwrap();

    let txn1 = SignedUserTransaction::mock();
    let txn2 = SignedUserTransaction::mock();
    let txn3 = SignedUserTransaction::mock();

    let msg_send1 = PeerMessage::new_transactions(
        network2.peer_id(),
        TransactionsMessage::new(vec![txn1.clone(), txn2.clone()]),
    );

    let msg_send2 = PeerMessage::new_transactions(
        network2.peer_id(),
        TransactionsMessage::new(vec![txn2.clone(), txn3.clone()]),
    );

    let msg_send3 = PeerMessage::new_transactions(
        network2.peer_id(),
        TransactionsMessage::new(vec![txn1.clone(), txn3.clone()]),
    );

    let mut receiver = network2.message_handler.channel();
    network1.service_ref.send_peer_message(msg_send1.clone());
    network1.service_ref.send_peer_message(msg_send2.clone());
    network1.service_ref.send_peer_message(msg_send3.clone());
    let msg_receive1 = receiver.next().await.unwrap();
    assert_eq!(msg_send1.notification, msg_receive1.notification);

    // msg2 only contains 1 txn after filter.
    let msg_receive2 = receiver.next().await.unwrap();
    assert_eq!(
        1,
        msg_receive2
            .notification
            .into_transactions()
            .unwrap()
            .txns
            .len()
    );

    //msg3 is empty after filter, so expect timeout error.
    let msg_receive3 = async_std::future::timeout(Duration::from_secs(1), receiver.next()).await;
    assert!(msg_receive3.is_err());
}

fn mock_block_info(total_difficulty: U256) -> BlockInfo {
    BlockInfo::new(
        HashValue::random(),
        total_difficulty,
        AccumulatorInfo::default(),
        AccumulatorInfo::default(),
    )
}

#[stest::test]
async fn test_event_broadcast() {
    let mut nodes = test_helper::build_network_cluster(3).await.unwrap();
    let node3 = nodes.pop().unwrap();
    let node2 = nodes.pop().unwrap();
    let node1 = nodes.pop().unwrap();

    let mut receiver1 = node1.message_handler.channel();
    let mut receiver2 = node2.message_handler.channel();
    let mut receiver3 = node3.message_handler.channel();

    let block = Block::new(BlockHeader::random(), BlockBody::new_empty());
    let notification = NotificationMessage::CompactBlock(Box::new(CompactBlockMessage::new(
        CompactBlock::new(block.clone()),
        //difficulty should > genesis block difficulty.
        mock_block_info(10.into()),
    )));
    node1.service_ref.broadcast(notification.clone());

    let msg_receive2 = receiver2.next().await.unwrap();
    assert_eq!(notification, msg_receive2.notification);

    let msg_receive3 = receiver3.next().await.unwrap();
    assert_eq!(notification, msg_receive3.notification);

    //repeat broadcast
    node2.service_ref.broadcast(notification.clone());

    let msg_receive1 = async_std::future::timeout(Duration::from_secs(1), receiver1.next()).await;
    assert!(msg_receive1.is_err());

    let msg_receive3 = async_std::future::timeout(Duration::from_secs(1), receiver3.next()).await;
    assert!(msg_receive3.is_err());

    print!("{:?}", node1.config.metrics.registry().unwrap().gather());
}

#[stest::test]
async fn test_send_announcement() {
    let node_config_1 = Arc::new(NodeConfig::random_for_test());
    let service1 = build_network_with_config(node_config_1.clone(), None)
        .await
        .unwrap();

    let nodes = vec![MultiaddrWithPeerId::new(
        node_config_1.network.listen(),
        service1.peer_id().into(),
    )];
    let mut node_config_2 = NodeConfig::random_for_test();
    node_config_2.network.seeds = nodes.into();
    let service2 = build_network_with_config(Arc::new(node_config_2), None)
        .await
        .unwrap();
    Delay::new(Duration::from_secs(2)).await;
    assert!(service2.service_ref.is_connected(service1.peer_id()).await);
    assert!(service1.service_ref.is_connected(service2.peer_id()).await);

    let ids = vec![HashValue::random()];
    let announcement =
        NotificationMessage::Announcement(Announcement::new(AnnouncementType::Txn, ids));
    let peer_message = PeerMessage::new(service2.peer_id(), announcement.clone());
    service1.service_ref.send_peer_message(peer_message);

    let mut receiver2 = service2.message_handler.channel();

    let msg_2 = receiver2.next().await.unwrap();
    assert_eq!(announcement, msg_2.notification);
}

#[stest::test]
async fn test_filter_protocol() {
    let node_config_1 = Arc::new(NodeConfig::random_for_test());
    let service1 = build_network_with_config(node_config_1.clone(), None)
        .await
        .unwrap();

    let nodes = vec![MultiaddrWithPeerId::new(
        node_config_1.network.listen(),
        service1.peer_id().into(),
    )];
    let mut node_config_2 = NodeConfig::random_for_test();
    node_config_2.network.seeds = nodes.into();
    let node_config_2 = Arc::new(node_config_2);
    let service2 = build_network_with_config(node_config_2.clone(), None)
        .await
        .unwrap();
    Delay::new(Duration::from_secs(2)).await;
    assert!(service2.service_ref.is_connected(service1.peer_id()).await);
    assert!(service1.service_ref.is_connected(service2.peer_id()).await);

    let nodes = vec![
        MultiaddrWithPeerId::new(node_config_1.network.listen(), service1.peer_id().into()),
        MultiaddrWithPeerId::new(node_config_2.network.listen(), service2.peer_id().into()),
    ];
    let mut node_config_3 = NodeConfig::random_for_test();
    node_config_3.network.seeds = nodes.into();
    node_config_3.network.unsupported_protocols = Some(vec![TXN_PROTOCOL_NAME.to_string()]);
    let service3 = build_network_with_config(Arc::new(node_config_3), None)
        .await
        .unwrap();
    Delay::new(Duration::from_secs(2)).await;
    assert!(service1.service_ref.is_connected(service3.peer_id()).await);
    assert!(service3.service_ref.is_connected(service1.peer_id()).await);

    let mut receiver2 = service2.message_handler.channel();
    let mut receiver3 = service3.message_handler.channel();

    let txns = vec![SignedUserTransaction::mock()];
    let notification = NotificationMessage::Transactions(TransactionsMessage::new(txns));
    service1.service_ref.broadcast(notification.clone());

    let msg_2 = receiver2.next().await.unwrap();
    assert_eq!(notification, msg_2.notification);

    let msg_3 = receiver3.next().await.unwrap();
    assert_eq!(
        ANNOUNCEMENT_PROTOCOL_NAME,
        msg_3.notification.protocol_name()
    );
}
