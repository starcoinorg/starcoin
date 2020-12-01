// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::net::{build_network_service, NetworkInner, SNetworkService};
    use crate::NetworkMessage;
    use crate::PeerEvent;
    use async_std::task;
    use bytes::Bytes;
    use config::{get_random_available_port, BuiltinNetworkID, NetworkConfig, NodeConfig};
    use crypto::hash::HashValue;
    use futures::channel::mpsc;
    use futures::{
        channel::mpsc::{UnboundedReceiver, UnboundedSender},
        stream::StreamExt,
    };
    use futures_timer::Delay;
    use network_api::messages::NotificationMessage;
    use network_api::Multiaddr;
    use network_p2p::{identity, DhtEvent, Event};
    use network_p2p::{NetworkConfiguration, NetworkWorker, NodeKeyConfig, Params, Secret};
    use network_p2p_types::{random_memory_addr, MultiaddrWithPeerId, PeerId};
    use std::borrow::Cow;
    use std::future::Future;
    use std::pin::Pin;
    use std::{thread, time::Duration};
    use types::startup_info::{ChainInfo, ChainStatus};

    const PROTOCOL_ID: &str = "starcoin";

    pub type NetworkComponent = (
        SNetworkService,
        UnboundedSender<NetworkMessage>,
        UnboundedReceiver<NetworkMessage>,
        UnboundedReceiver<PeerEvent>,
        UnboundedSender<()>,
        NetworkConfig,
    );

    async fn build_test_network_pair_not_wait() -> (NetworkComponent, NetworkComponent) {
        let (service1, service2) = build_test_network_pair();
        let from_peer_id = service1.0.identify().clone();
        let to_peer_id = service2.0.identify().clone();
        thread::sleep(Duration::from_secs(2));
        assert!(service1.0.is_connected(to_peer_id.clone()).await.unwrap());
        assert!(service2.0.is_connected(from_peer_id).await.unwrap());
        let to_peer_id_str = format!(
            "{}/p2p/{}",
            service2.5.listen.to_string(),
            to_peer_id.to_base58()
        );
        debug!("to peer : {:?}", to_peer_id_str);
        service1.0.add_peer(to_peer_id_str).unwrap();
        (service1, service2)
    }

    fn gen_network_inner() -> NetworkInner {
        let node_config = NodeConfig::random_for_test();

        let chain_info = ChainInfo::new(
            node_config.net().chain_id(),
            HashValue::default(),
            ChainStatus::random(),
        );
        let mut cfg = node_config.network;
        cfg.listen = format!(
            "/ip4/{}/tcp/{}",
            "127.0.0.1".to_string(),
            get_random_available_port() as u16
        )
        .parse()
        .unwrap();

        let config = NetworkConfiguration {
            listen_addresses: vec![cfg.listen.clone()],
            boot_nodes: cfg.seeds.clone(),
            node_key: {
                let secret = identity::ed25519::SecretKey::from_bytes(
                    &mut cfg.network_keypair().private_key.to_bytes(),
                )
                .unwrap();
                NodeKeyConfig::Ed25519(Secret::Input(secret))
            },
            protocols: NotificationMessage::protocols(),
            ..NetworkConfiguration::default()
        };

        let protocol = network_p2p::ProtocolId::from(PROTOCOL_ID);
        let worker = NetworkWorker::new(Params::new(config, protocol, chain_info, None)).unwrap();
        let service = worker.service().clone();
        NetworkInner::new(service)
    }

    async fn test_handle_event(event: Event) {
        let network_inner: NetworkInner = gen_network_inner();
        let (net_tx, _rx) = mpsc::unbounded::<NetworkMessage>();
        let (event_tx, _event_rx) = mpsc::unbounded::<PeerEvent>();
        assert!(network_inner
            .handle_network_receive_inner(event, net_tx, event_tx)
            .await
            .is_ok());
    }

    fn build_test_network_pair() -> (NetworkComponent, NetworkComponent) {
        let mut l = build_test_network_services(2).into_iter();
        let a = l.next().unwrap();
        let b = l.next().unwrap();
        (a, b)
    }

    fn build_test_network_services(
        num: usize,
    ) -> Vec<(
        SNetworkService,
        UnboundedSender<NetworkMessage>,
        UnboundedReceiver<NetworkMessage>,
        UnboundedReceiver<PeerEvent>,
        UnboundedSender<()>,
        NetworkConfig,
    )> {
        let mut result: Vec<(
            SNetworkService,
            UnboundedSender<NetworkMessage>,
            UnboundedReceiver<NetworkMessage>,
            UnboundedReceiver<PeerEvent>,
            UnboundedSender<()>,
            NetworkConfig,
        )> = Vec::with_capacity(num);
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
                    result[0].0.identify().clone(),
                ));
            }
            let node_config = NodeConfig::random_for_test();
            let mut config = node_config.network.clone();

            config.listen = random_memory_addr();
            config.seeds = boot_nodes;

            info!("listen:{:?},boots {:?}", config.listen, config.seeds);
            if first_addr.is_none() {
                first_addr = Some(config.listen.clone());
            }
            let mut protocols = NotificationMessage::protocols();
            protocols.push(TEST_NOTIF_PROTOCOL_NAME.into());
            let server = build_network_service(chain_info.clone(), &config, protocols, None);
            result.push({
                let c: NetworkComponent = (
                    server.0,
                    server.1,
                    server.2,
                    server.3,
                    server.4,
                    config.clone(),
                );
                c
            });
        }
        result
    }

    const TEST_NOTIF_PROTOCOL_NAME: &str = "/test_notif";
    #[test]
    fn test_send_receive_1() {
        ::logger::init_for_test();
        //let mut rt = Builder::new().core_threads(1).build().unwrap();
        let (
            (service1, tx1, rx1, _event_rx1, close_tx1, _),
            (service2, tx2, _rx2, _event_rx2, close_tx2, _),
        ) = build_test_network_pair();
        let msg_peer_id_1 = service1.identify().clone();
        let msg_peer_id_2 = service2.identify().clone();
        // Once sender has been droped, the select_all will return directly. clone it to prevent it.
        let _tx22 = tx2.clone();
        let _tx11 = tx1.clone();
        //wait the network started.
        thread::sleep(Duration::from_secs(1));
        let sender_fut = async move {
            let mut continue_loop = true;
            let mut count: i32 = 0;
            while continue_loop {
                if count == 1000 {
                    continue_loop = false;
                }
                debug!("count is {}", count);
                count += 1;
                Delay::new(Duration::from_millis(1)).await;
                let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();

                match if count % 2 == 0 {
                    tx2.unbounded_send(NetworkMessage {
                        peer_id: msg_peer_id_1.clone(),
                        protocol_name: std::borrow::Cow::Borrowed(TEST_NOTIF_PROTOCOL_NAME),
                        data: random_bytes,
                    })
                } else {
                    tx1.unbounded_send(NetworkMessage {
                        peer_id: msg_peer_id_2.clone(),
                        protocol_name: std::borrow::Cow::Borrowed(TEST_NOTIF_PROTOCOL_NAME),
                        data: random_bytes,
                    })
                } {
                    Ok(()) => debug!("ok"),
                    Err(e) => warn!("err: {:?}", e),
                }
            }
        };
        let receive_fut = async move {
            let mut rx1 = rx1.fuse();
            loop {
                futures::select! {
                    _message = rx1.select_next_some()=>{
                        info!("receive message ");
                    },
                    complete => {
                        info!("complete");
                        break;
                    }
                }
            }
        };
        task::spawn(receive_fut);
        task::spawn(sender_fut);

        let task = async move {
            Delay::new(Duration::from_secs(6)).await;
            let _ = close_tx1.unbounded_send(());
            let _ = close_tx2.unbounded_send(());
        };
        task::block_on(task);
    }

    #[test]
    fn test_connected_nodes() {
        ::logger::init_for_test();

        let (service1, _service2) = build_test_network_pair();
        thread::sleep(Duration::from_secs(2));
        let fut = async move {
            assert_eq!(
                service1
                    .0
                    .is_connected(_service2.0.identify().clone())
                    .await
                    .unwrap(),
                true
            );
            // assert_eq!(
            //     AccountAddress::from_str(&hex::encode(service1.0.identify())).unwrap(),
            //     service1.0.identify()
            // );
        };
        task::block_on(fut);
    }

    #[stest::test]
    async fn test_network_broadcast_message() {
        let (mut service1, mut service2) = build_test_network_pair_not_wait().await;
        let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();
        service1
            .0
            .broadcast_message(TEST_NOTIF_PROTOCOL_NAME.into(), random_bytes.clone())
            .await;
        let mut receiver = service2.2.select_next_some();
        let response = futures::future::poll_fn(move |cx| Pin::new(&mut receiver).poll(cx)).await;
        assert_eq!(response.data, random_bytes);
    }

    #[stest::test]
    async fn test_network_exist_notify_proto() {
        let service: NetworkComponent = build_test_network_services(1).into_iter().next().unwrap();
        assert!(
            service
                .0
                .exist_notif_proto(TEST_NOTIF_PROTOCOL_NAME.into())
                .await
        );
    }

    #[stest::test]
    async fn test_event_dht() {
        let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();
        let event = Event::Dht(DhtEvent::ValuePut(random_bytes.clone().into()));
        test_handle_event(event).await;
    }

    #[stest::test]
    async fn test_event_notify_open() {
        let event = Event::NotificationStreamOpened {
            remote: PeerId::random(),
            info: Box::new(ChainInfo::random()),
        };
        test_handle_event(event).await;
    }

    #[stest::test]
    async fn test_event_notify_close() {
        let event = Event::NotificationStreamClosed {
            remote: PeerId::random(),
        };
        test_handle_event(event).await;
    }

    #[stest::test]
    async fn test_event_notify_receive() {
        let mut data = Vec::new();
        data.push(Bytes::from(&b"hello"[..]));
        let event = Event::NotificationsReceived {
            remote: PeerId::random(),
            protocol: TEST_NOTIF_PROTOCOL_NAME.into(),
            messages: data,
        };
        test_handle_event(event).await;
    }

    //TOD FIXME  provider a shutdown network method, shutdown first, then start.
    #[ignore]
    #[stest::test]
    fn test_reconnected_nodes() {
        let protocols: Vec<Cow<'static, str>> = vec![TEST_NOTIF_PROTOCOL_NAME.into()];
        let mut node_config1 = NodeConfig::random_for_test().network;
        node_config1.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_random_available_port())
            .parse()
            .unwrap();

        let chain_info = ChainInfo::new(
            BuiltinNetworkID::Test.chain_id(),
            HashValue::random(),
            ChainStatus::random(),
        );

        let (service1, _net_tx1, _net_rx1, _event_rx1, _command_tx1) =
            build_network_service(chain_info.clone(), &node_config1, protocols.clone(), None);

        thread::sleep(Duration::from_secs(1));

        let mut node_config2 = NodeConfig::random_for_test().network;
        let addr1_hex = service1.identify().to_base58();
        let seed: MultiaddrWithPeerId = format!("{}/p2p/{}", &node_config1.listen, addr1_hex)
            .parse()
            .unwrap();
        node_config2.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_random_available_port())
            .parse()
            .unwrap();
        node_config2.seeds = vec![seed.clone()];
        let (service2, net_tx2, net_rx2, event_rx2, command_tx2) =
            build_network_service(chain_info.clone(), &node_config2, protocols.clone(), None);

        thread::sleep(Duration::from_secs(1));

        let mut node_config3 = NodeConfig::random_for_test().network;
        node_config3.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_random_available_port())
            .parse()
            .unwrap();
        node_config3.seeds = vec![seed];
        let (service3, net_tx3, net_rx3, event_rx3, command_tx3) =
            build_network_service(chain_info.clone(), &node_config3, protocols.clone(), None);

        thread::sleep(Duration::from_secs(1));

        let service1_clone = service1.clone();
        let fut = async move {
            assert_eq!(
                service1_clone
                    .is_connected(service2.identify().clone())
                    .await
                    .unwrap(),
                true
            );
            assert_eq!(
                service1_clone
                    .is_connected(service3.identify().clone())
                    .await
                    .unwrap(),
                true
            );

            drop(service2);
            drop(service3);

            drop(net_tx2);
            drop(net_rx2);
            drop(event_rx2);

            drop(net_tx3);
            drop(net_rx3);
            drop(event_rx3);

            command_tx2.close_channel();
            command_tx3.close_channel();
            Delay::new(Duration::from_secs(1)).await;
        };
        task::block_on(fut);

        thread::sleep(Duration::from_secs(1));

        let (service2, _net_tx2, _net_rx2, _event_tx2, _command_tx2) =
            build_network_service(chain_info.clone(), &node_config2, protocols.clone(), None);

        thread::sleep(Duration::from_secs(1));

        let (service3, _net_tx3, _net_rx3, _event_rx3, _command_tx3) =
            build_network_service(chain_info, &node_config3, protocols.clone(), None);

        thread::sleep(Duration::from_secs(1));

        let fut = async move {
            assert_eq!(
                service1
                    .is_connected(service2.identify().clone())
                    .await
                    .unwrap(),
                true
            );
            assert_eq!(
                service1
                    .is_connected(service3.identify().clone())
                    .await
                    .unwrap(),
                true
            );
        };
        task::block_on(fut);
    }
}
