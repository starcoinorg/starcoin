// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::service::NetworkStateInfo;
    use crate::{Event, Multiaddr, NodeKeyConfig, ProtocolId, Secret};
    use crate::{NetworkConfiguration, NetworkWorker, Params};
    use crypto::HashValue;
    use futures::stream::StreamExt;
    use libp2p::identity;
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::Runtime;

    const PROTOCOL_NAME: &[u8] = b"/starcoin/notify/1";

    #[stest::test(timeout = 5)]
    #[allow(clippy::string_lit_as_bytes)]
    fn test_notify() {
        let mut rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let protocol = ProtocolId::from(b"stargate".as_ref());
        let config1 = generate_config(vec![]);

        let worker1 = NetworkWorker::new(Params::new(config1.clone(), protocol.clone())).unwrap();
        let service1 = worker1.service().clone();
        let mut stream1 = service1.event_stream();

        handle.spawn(worker1);

        let addr1_hex = service1.peer_id().to_base58();
        let seed: Multiaddr = format!(
            "{}/p2p/{}",
            &config1.listen_addresses.get(0).expect("should have"),
            addr1_hex
        )
        .parse()
        .unwrap();
        let config2 = generate_config(vec![seed]);

        let worker2 = NetworkWorker::new(Params::new(config2.clone(), protocol)).unwrap();
        let service2 = worker2.service().clone();
        let mut stream2 = service2.event_stream();

        handle.spawn(worker2);

        let data = vec![1, 2, 3, 4];
        let data_clone = data.clone();
        let addr1 = service1.peer_id().clone();

        info!(
            "first peer address is {:?} id is {:?},second peer address is {:?} id is {:?}",
            config1.listen_addresses,
            service1.local_peer_id(),
            config2.listen_addresses,
            service2.local_peer_id()
        );

        let fut = async move {
            while let Some(event) = stream2.next().await {
                match event {
                    Event::NotificationStreamOpened { remote, info } => {
                        info!("open stream from {},info is {:?}", remote, info);
                        let result = service2.get_address(remote.clone()).await;
                        info!("remote {} address is {:?}", remote, result);
                        service2.write_notification(
                            addr1.clone(),
                            PROTOCOL_NAME.into(),
                            data_clone.clone(),
                        );
                    }
                    _ => {
                        info!("event is {:?}", event);
                    }
                }
            }
        };

        handle.spawn(fut);

        let fut = async move {
            while let Some(event) = stream1.next().await {
                match event {
                    Event::NotificationsReceived {
                        remote,
                        protocol_name,
                        mut messages,
                    } => {
                        let msg = messages.remove(0).to_vec();
                        info!("receive message {:?} from {} ", msg, remote);
                        assert_eq!(protocol_name.as_ref(), PROTOCOL_NAME);
                        assert_eq!(msg, data);
                        break;
                    }
                    Event::NotificationStreamOpened { remote, info } => {
                        info!("open stream from {},info is {:?}", remote, info);
                        let result = service1.get_address(remote.clone()).await;
                        info!("remote {} address is {:?}", remote, result);
                    }
                    _ => {
                        info!("event is {:?}", event);
                    }
                }
            }
        };

        rt.block_on(fut);
    }

    #[test]
    #[allow(clippy::string_lit_as_bytes)]
    fn test_handshake_fail() {
        ::logger::init_for_test();

        let mut rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let protocol = ProtocolId::from(b"stargate".as_ref());
        let config1 = generate_config(vec![]);

        let worker1 = NetworkWorker::new(Params::new(config1.clone(), protocol.clone())).unwrap();
        let service1 = worker1.service().clone();
        let mut stream = service1.event_stream();

        handle.spawn(worker1);

        let addr1_hex = service1.peer_id().to_base58();
        let seed: Multiaddr = format!(
            "{}/p2p/{}",
            &config1.listen_addresses.get(0).expect("should have"),
            addr1_hex
        )
        .parse()
        .unwrap();
        let mut config2 = generate_config(vec![seed]);
        config2.genesis_hash = HashValue::random();

        let worker2 = NetworkWorker::new(Params::new(config2, protocol)).unwrap();
        let service2 = worker2.service().clone();

        handle.spawn(worker2);

        thread::sleep(Duration::from_secs(1));

        info!(
            "first peer is {:?},second peer is {:?}",
            service1.peer_id(),
            service2.peer_id()
        );
        let fut = async move {
            while let Some(event) = stream.next().await {
                match event {
                    Event::NotificationStreamClosed { remote } => {
                        info!("handshake failed from {}", remote);
                        break;
                    }
                    _ => {
                        info!("event is {:?}", event);
                    }
                }
            }
        };

        rt.block_on(fut);
    }

    fn generate_config(boot_nodes: Vec<Multiaddr>) -> NetworkConfiguration {
        let mut config = NetworkConfiguration::default();
        let listen = format!("/ip4/127.0.0.1/tcp/{}", sg_config::get_available_port());
        config.listen_addresses = vec![listen.parse().expect("Failed to parse network config")];
        let keypair = sg_config::gen_keypair();
        config.node_key = {
            let secret =
                identity::ed25519::SecretKey::from_bytes(&mut keypair.private_key.to_bytes())
                    .unwrap();
            NodeKeyConfig::Ed25519(Secret::Input(secret))
        };
        config.boot_nodes = boot_nodes;
        config.protocols.push(PROTOCOL_NAME.into());
        config
    }
}
