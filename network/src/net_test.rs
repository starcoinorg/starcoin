// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use config::get_available_port;
    use futures::{
        channel::mpsc::{UnboundedReceiver, UnboundedSender},
        stream::StreamExt,
    };
    use std::{thread, time::Duration};
    use tokio::runtime::{Handle, Runtime};

    use futures_timer::Delay;

    use network_p2p::PeerId;
    use types::peer_info::PeerId as SPeerId;

    use crate::{helper::convert_boot_nodes, PeerEvent};

    use crate::net::{build_network_service, SNetworkService};

    use crate::messages::NetworkMessage;

    pub type NetworkComponent = (
        SNetworkService,
        UnboundedSender<NetworkMessage>,
        UnboundedReceiver<NetworkMessage>,
        UnboundedReceiver<PeerEvent>,
        UnboundedSender<()>,
    );

<<<<<<< HEAD
    fn build_test_network_pair(
        host: String,
        handle: Handle,
    ) -> (NetworkComponent, NetworkComponent) {
        let mut l = build_test_network_services(2, host, 50400, handle).into_iter();
=======
    fn build_test_network_pair(handle: Handle) -> (NetworkComponent, NetworkComponent) {
        let mut l = build_test_network_services(2, get_available_port(), handle).into_iter();
>>>>>>> 4ce8e5e10a690f9953577c709dc4d914cda29180
        let a = l.next().unwrap();
        let b = l.next().unwrap();
        (a, b)
    }

    fn build_test_network_services(
        num: usize,
        host: String,
        base_port: u16,
        handle: Handle,
    ) -> Vec<(
        SNetworkService,
        UnboundedSender<NetworkMessage>,
        UnboundedReceiver<NetworkMessage>,
        UnboundedReceiver<PeerEvent>,
        UnboundedSender<()>,
    )> {
        let mut result: Vec<(
            SNetworkService,
            UnboundedSender<NetworkMessage>,
            UnboundedReceiver<NetworkMessage>,
            UnboundedReceiver<PeerEvent>,
            UnboundedSender<()>,
        )> = Vec::with_capacity(num);
        let mut first_addr = None::<String>;
        for index in 0..num {
            let mut boot_nodes = Vec::new();

            if let Some(first_addr) = first_addr.as_ref() {
                boot_nodes.push(format!(
                    "{}/p2p/{}",
                    first_addr,
                    result[0].0.identify().to_base58()
                ));
            }
            let mut config = config::NetworkConfig::random_for_test();

            config.listen = format!("/ip4/{}/tcp/{}", host, base_port + index as u16);
            config.seeds = boot_nodes;

            info!("listen:{:?},boots {:?}", config.listen, config.seeds);
            if first_addr.is_none() {
                first_addr = Some(config.listen.clone().parse().unwrap());
            }

            let server = build_network_service(&config, handle.clone());
            result.push({
                let c: NetworkComponent = server;
                c
            });
        }
        result
    }

    #[test]
    fn test_send_receive_1() {
        ::logger::init_for_test();
        //let mut rt = Builder::new().core_threads(1).build().unwrap();
        let mut rt = Runtime::new().unwrap();

        let executor = rt.handle();
        let (
            (service1, tx1, rx1, _event_rx1, close_tx1),
            (service2, tx2, _rx2, _event_rx2, close_tx2),
        ) = build_test_network_pair("127.0.0.1".to_string(), executor.clone());
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
                info!("count is {}", count);
                count = count + 1;
                Delay::new(Duration::from_millis(1)).await;
                let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();

                match if count % 2 == 0 {
                    tx2.unbounded_send(NetworkMessage {
                        peer_id: SPeerId::from(msg_peer_id_1.clone()),
                        data: random_bytes,
                    })
                } else {
                    tx1.unbounded_send(NetworkMessage {
                        peer_id: SPeerId::from(msg_peer_id_2.clone()),
                        data: random_bytes,
                    })
                } {
                    Ok(()) => info!("ok"),
                    Err(_e) => warn!("err"),
                }
            }
        };
        let receive_fut = async move {
            let mut rx1 = rx1.fuse();
            loop {
                futures::select! {
                    message = rx1.select_next_some()=>{
                        info!("recevie message {:?}",message);
                    },
                    complete => {
                        info!("complete");
                        break;
                    }
                }
            }
        };
        executor.spawn(receive_fut);
        rt.handle().spawn(sender_fut);

        let task = async move {
            Delay::new(Duration::from_secs(6)).await;
            let _ = close_tx1.unbounded_send(());
            let _ = close_tx2.unbounded_send(());
        };
        rt.block_on(task);
    }

    #[test]
    fn test_send_receive_2() {
        ::logger::init_for_test();

        let rt = Runtime::new().unwrap();
        let executor = rt.handle();
        let (
            (service1, _tx1, rx1, _event_rx1, _close_tx1),
            (service2, _tx2, _rx2, _event_rx2, _close_tx2),
        ) = build_test_network_pair("127.0.0.1".to_string(), executor.clone());
        let msg_peer_id = service1.identify().clone();
        let receive_fut = async move {
            let mut rx1 = rx1.fuse();
            loop {
                futures::select! {
                    message = rx1.select_next_some()=>{
                        info!("recevie message {:?}",message);
                    },
                    complete => {
                        info!("complete");
                        break;
                    }
                }
            }
        };

        executor.clone().spawn(receive_fut);

        //wait the network started.
        thread::sleep(Duration::from_secs(1));

        for _x in 0..1000 {
            let random_bytes: Vec<u8> = (0..10240).map(|_| rand::random::<u8>()).collect();
            let service2_clone = service2.clone();

            let peer_id = msg_peer_id.clone();
            let fut = async move {
                assert_eq!(
                    service2_clone.is_connected(peer_id.clone()).await.unwrap(),
                    true
                );

                service2_clone
                    .send_message(peer_id, random_bytes)
                    .await
                    .unwrap();
            };
            executor.spawn(fut);
        }
        thread::sleep(Duration::from_secs(3));
    }

    #[test]
    fn test_connected_nodes() {
        ::logger::init_for_test();

        let mut _rt = Runtime::new().unwrap();
        let (service1, _service2) =
            build_test_network_pair("127.0.0.1".to_string(), _rt.handle().clone());
        thread::sleep(Duration::from_secs(1));
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
        _rt.block_on(fut);
    }

    fn generate_peer_address() -> String {
        PeerId::random().to_base58()
    }

    #[test]
    fn test_boot_nodes() {
        let mut boot_nodes = Vec::new();

        boot_nodes
            .push(format!("/ip4/127.0.0.1/tcp/5000/p2p/{:}", generate_peer_address()).to_string());
        boot_nodes.iter().for_each(|x| println!("{}", x));

        let boot_nodes = convert_boot_nodes(boot_nodes);
        boot_nodes.iter().for_each(|x| println!("{}", x));
    }
}
