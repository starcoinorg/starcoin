// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::{Event, Multiaddr, NodeKeyConfig, PeerId, ProtocolId, Secret};
    use crate::{NetworkConfiguration, NetworkWorker, Params};
    use crypto::HashValue;
    use futures::stream::StreamExt;
    use libp2p::identity;
    use std::str::FromStr;
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::{Handle, Runtime};
    use tokio::task;

    const PROTOCOL_NAME: &[u8] = b"/starcoin/notify/1";

    #[test]
    fn test_notify() {
        ::logger::init_for_test();

        let mut rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let protocol = ProtocolId::from("stargate".as_bytes());
        let config1 = generate_config(vec![]);

        let worker1 = NetworkWorker::new(Params::new(config1.clone(), protocol.clone())).unwrap();
        let service1 = worker1.service().clone();
        let mut stream = service1.event_stream();
        service1.register_notifications_protocol(PROTOCOL_NAME);

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

        let worker2 = NetworkWorker::new(Params::new(config2.clone(), protocol.clone())).unwrap();
        let service2 = worker2.service().clone();
        service2.register_notifications_protocol(PROTOCOL_NAME);

        handle.spawn(worker2);

        thread::sleep(Duration::from_secs(1));

        let data = vec![1, 2, 3, 4];
        service2.write_notification(
            service1.peer_id().clone(),
            PROTOCOL_NAME.into(),
            data.clone(),
        );

        info!(
            "first peer is {:?},second peer is {:?}",
            service1.peer_id(),
            service2.peer_id()
        );
        let fut = async move {
            while let Some(event) = stream.next().await {
                match event {
                    Event::NotificationsReceived {
                        remote,
                        mut messages,
                    } => {
                        let msg = messages.remove(0).to_vec();
                        info!("receive message {:?} from {} ", msg, remote);
                        assert_eq!(msg, data);
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

    #[test]
    fn test_handshake_fail() {
        ::logger::init_for_test();

        let mut rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let protocol = ProtocolId::from("stargate".as_bytes());
        let config1 = generate_config(vec![]);

        let worker1 = NetworkWorker::new(Params::new(config1.clone(), protocol.clone())).unwrap();
        let service1 = worker1.service().clone();
        let mut stream = service1.event_stream();
        service1.register_notifications_protocol(PROTOCOL_NAME);

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

        let worker2 = NetworkWorker::new(Params::new(config2.clone(), protocol.clone())).unwrap();
        let service2 = worker2.service().clone();
        service2.register_notifications_protocol(PROTOCOL_NAME);

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
        config.listen_addresses = vec![listen
            .clone()
            .parse()
            .expect("Failed to parse network config")];
        let keypair = sg_config::gen_keypair();
        config.node_key = {
            let secret =
                identity::ed25519::SecretKey::from_bytes(&mut keypair.private_key.to_bytes())
                    .unwrap();
            NodeKeyConfig::Ed25519(Secret::Input(secret))
        };
        config.boot_nodes = boot_nodes;
        config
    }
}
