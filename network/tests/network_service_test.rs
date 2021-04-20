// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use async_std::task;
use futures::stream::StreamExt;
use futures_timer::Delay;
use network_api::messages::{
    Announcement, AnnouncementType, CompactBlockMessage, NotificationMessage, PeerMessage,
    TransactionsMessage, ANNOUNCEMENT_PROTOCOL_NAME,
};
use network_api::{Multiaddr, NetworkService};
use network_p2p_types::MultiaddrWithPeerId;
use starcoin_config::{BuiltinNetworkID, NetworkConfig, NodeConfig};
use starcoin_crypto::hash::HashValue;
use starcoin_logger::prelude::*;
use starcoin_network::{build_network_worker, NetworkActorService};
use starcoin_types::block::{AccumulatorInfo, Block, BlockBody, BlockHeader, BlockInfo};
use starcoin_types::cmpact_block::CompactBlock;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::U256;
use std::borrow::Cow;
use std::sync::Arc;
use std::{thread, time::Duration};
use test_helper::network::MockPeerMessageHandler;

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
        let (_peer_info, worker) =
            build_network_worker(&node_config.network, chain_info.clone(), protocols, None)
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
        assert_eq!(service1.0.is_connected(*service2.0.peer_id()).await, true);
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
            CompactBlock::new(
                Block::new(BlockHeader::random(), BlockBody::new_empty()),
                vec![],
            ),
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
        CompactBlockMessage::new(
            CompactBlock::new(block.clone(), vec![]),
            mock_block_info(1.into()),
        ),
    );

    let msg_send2 = PeerMessage::new_compact_block(
        network2.peer_id(),
        CompactBlockMessage::new(
            CompactBlock::new(block.clone(), vec![]),
            mock_block_info(1.into()),
        ),
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
        CompactBlock::new(block.clone(), vec![]),
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
}

fn build_network_service_with_protocol(
    node_config: Arc<NodeConfig>,
    chain_info: &ChainInfo,
    broadcast_protocols: Vec<Cow<'static, str>>,
) -> (NetworkActorService, PeerInfo, MockPeerMessageHandler) {
    let (peer_info, service) = build_network_worker(
        &node_config.network,
        chain_info.clone(),
        broadcast_protocols,
        None,
    )
    .unwrap();
    let peer_message_handler = MockPeerMessageHandler::default();
    let network = NetworkActorService::new_with_network_for_test(
        node_config,
        peer_message_handler.clone(),
        service,
        peer_info.clone(),
    )
    .unwrap();
    (network, peer_info, peer_message_handler)
}

#[stest::test]
async fn test_send_announcement() {
    let chain_info = ChainInfo::new(
        BuiltinNetworkID::Test.chain_id(),
        HashValue::random(),
        ChainStatus::random(),
    );
    let node_config_1 = Arc::new(NodeConfig::random_for_test());
    let (mut service1, peer_info_1, _peer_message_handler_1) = build_network_service_with_protocol(
        node_config_1.clone(),
        &chain_info,
        NotificationMessage::protocols(),
    );

    let nodes = vec![MultiaddrWithPeerId::new(
        node_config_1.network.listen(),
        peer_info_1.peer_id().into(),
    )];
    let mut node_config_2 = NodeConfig::random_for_test();
    node_config_2.network.seeds = nodes.into();
    let node_config_2 = Arc::new(node_config_2);
    let (_service2, peer_info_2, peer_message_handler_2) = build_network_service_with_protocol(
        node_config_2,
        &chain_info,
        vec![ANNOUNCEMENT_PROTOCOL_NAME.into()],
    );

    thread::sleep(Duration::from_secs(2));

    let ids = vec![HashValue::random()];
    let announcement =
        NotificationMessage::Announcement(Announcement::new(AnnouncementType::Txn, ids));
    service1.send_peer_message_for_test(peer_info_2.peer_id(), announcement.clone());

    let mut receiver2 = peer_message_handler_2.channel();

    let msg_receive2 = receiver2.next().await.unwrap();
    assert_eq!(announcement, msg_receive2.notification);
}

#[stest::test]
async fn test_filter_protocol() {
    let chain_info = ChainInfo::new(
        BuiltinNetworkID::Test.chain_id(),
        HashValue::random(),
        ChainStatus::random(),
    );
    let node_config_1 = Arc::new(NodeConfig::random_for_test());
    let (mut service1, peer_info_1, _peer_message_handler_1) = build_network_service_with_protocol(
        node_config_1.clone(),
        &chain_info,
        NotificationMessage::protocols(),
    );

    let nodes = vec![MultiaddrWithPeerId::new(
        node_config_1.network.listen(),
        peer_info_1.peer_id().into(),
    )];
    let mut node_config_2 = NodeConfig::random_for_test();
    node_config_2.network.seeds = nodes.into();
    let node_config_2 = Arc::new(node_config_2);
    let (_service2, peer_info_2, peer_message_handler_2) = build_network_service_with_protocol(
        node_config_2.clone(),
        &chain_info,
        NotificationMessage::protocols(),
    );

    thread::sleep(Duration::from_secs(2));

    let nodes = vec![
        MultiaddrWithPeerId::new(node_config_1.network.listen(), peer_info_1.peer_id().into()),
        MultiaddrWithPeerId::new(node_config_2.network.listen(), peer_info_2.peer_id().into()),
    ];
    let mut node_config_3 = NodeConfig::random_for_test();
    node_config_3.network.seeds = nodes.into();
    let node_config_3 = Arc::new(node_config_3);
    let (_service3, _peer_info_3, peer_message_handler_3) = build_network_service_with_protocol(
        node_config_3,
        &chain_info,
        vec![ANNOUNCEMENT_PROTOCOL_NAME.into()],
    );

    thread::sleep(Duration::from_secs(2));

    let txns = vec![SignedUserTransaction::mock()];
    let notification = NotificationMessage::Transactions(TransactionsMessage::new(txns));
    service1.broadcast_for_test(notification.clone());

    let mut receiver2 = peer_message_handler_2.channel();
    let msg_receive2 = receiver2.next().await.unwrap();
    assert_eq!(notification, msg_receive2.notification);

    let mut receiver3 = peer_message_handler_3.channel();
    let msg_receive3 = receiver3.next().await.unwrap();
    assert_eq!(
        ANNOUNCEMENT_PROTOCOL_NAME,
        msg_receive3.notification.protocol_name()
    );
}
