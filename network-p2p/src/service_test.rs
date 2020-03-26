// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::{Event, NodeKeyConfig, PeerId, ProtocolId, Secret};
    use crate::{NetworkConfiguration, NetworkWorker, Params};
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

        let mut config1 = sg_config::NetworkConfig::random_for_test();
        config1.listen = format!("/ip4/127.0.0.1/tcp/{}", sg_config::get_available_port());

        let protocol = ProtocolId::from("stargate".as_bytes());
        let worker1 = NetworkWorker::new(Params::new(
            convert_config(config1.clone()),
            protocol.clone(),
        ))
        .unwrap();
        let service1 = worker1.service().clone();
        service1.register_notifications_protocol(PROTOCOL_NAME);

        handle.spawn(worker1);

        let mut config2 = sg_config::NetworkConfig::random_for_test();
        config2.listen = format!("/ip4/127.0.0.1/tcp/{}", sg_config::get_available_port());
        let addr1_hex = service1.peer_id().to_base58();
        let seed = format!("{}/p2p/{}", &config1.listen, addr1_hex);
        config2.seeds = vec![seed];
        let protocol = ProtocolId::from("stargate".as_bytes());
        let worker2 = NetworkWorker::new(Params::new(
            convert_config(config2.clone()),
            protocol.clone(),
        ))
        .unwrap();
        let service2 = worker2.service().clone();
        service2.register_notifications_protocol(PROTOCOL_NAME);

        handle.spawn(worker2);

        thread::sleep(Duration::from_secs(1));

        let mut stream = service1.event_stream();

        service2.write_notification(
            service1.peer_id().clone(),
            PROTOCOL_NAME.into(),
            vec![1, 2, 3, 4],
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
                        info!("len is {:?}", messages.remove(0));
                    }
                    _ => {
                        info!("event is {:?}", event);
                    }
                }
            }
        };

        rt.block_on(fut);
    }

    fn convert_config(cfg: sg_config::NetworkConfig) -> NetworkConfiguration {
        let config = NetworkConfiguration {
            listen_addresses: vec![cfg.listen.parse().expect("Failed to parse network config")],
            boot_nodes: convert_boot_nodes(cfg.seeds.clone()),
            node_key: {
                let secret = identity::ed25519::SecretKey::from_bytes(
                    &mut cfg.network_keypair().private_key.to_bytes(),
                )
                .unwrap();
                NodeKeyConfig::Ed25519(Secret::Input(secret))
            },
            ..NetworkConfiguration::default()
        };
        config
    }

    fn convert_boot_nodes(boot_nodes: Vec<String>) -> Vec<String> {
        boot_nodes
            .iter()
            .map(|x| {
                let dx = x.rfind("/").expect("Failed to parse boot nodes");

                let peer_address = &x[dx + 1..];
                let addr = &x[..dx];
                let peer_id =
                    PeerId::from_str(peer_address).expect("Failed to parse account address");
                format!("{:}/{:}", addr, peer_id).to_string()
            })
            .clone()
            .collect()
    }
}
