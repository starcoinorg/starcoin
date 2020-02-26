// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{convert_account_address_to_peer_id, convert_peer_id_to_account_address, helper::convert_boot_nodes, PayloadMsg};
use crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};

use crate::messages::{Message, NetworkMessage};
use futures::{
    future,
    stream::{self, Stream},
    sync::{
        mpsc,
        oneshot::{self, Canceled, Sender},
    },
    try_ready, Async, Future,
};

use config::NetworkConfig;
use network_libp2p::{
    identity, start_service, NetworkConfiguration, NodeKeyConfig, Secret, Service as Libp2pService,
    ServiceEvent,
};
use parity_codec::alloc::collections::HashSet;
use parking_lot::Mutex;
use std::{collections::HashMap, io, sync::Arc, thread};
use tokio::prelude::task::AtomicTask;
use types::account_address::AccountAddress;

#[derive(Clone)]
pub struct NetworkService {
    pub libp2p_service: Arc<Mutex<Libp2pService>>,
    acks: Arc<Mutex<HashMap<u128, Sender<()>>>>,
}

pub fn build_network_service(
    cfg: &NetworkConfig,
    key_pair: Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
) -> (
    NetworkService,
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
    oneshot::Sender<()>,
) {
    let config = NetworkConfiguration {
        listen_addresses: vec![cfg.listen.parse().expect("Failed to parse network config")],
        boot_nodes: convert_boot_nodes(cfg.seeds.clone()),
        node_key: {
            let secret =
                identity::ed25519::SecretKey::from_bytes(&mut key_pair.private_key.to_bytes())
                    .unwrap();
            NodeKeyConfig::Ed25519(Secret::Input(secret))
        },
        ..NetworkConfiguration::default()
    };
    NetworkService::new(config)
}

fn build_libp2p_service(cfg: NetworkConfiguration) -> Result<Arc<Mutex<Libp2pService>>, io::Error> {
    let protocol = network_libp2p::ProtocolId::from("stargate".as_bytes());
    match start_service(protocol, cfg) {
        Ok((srv, _)) => Ok(Arc::new(Mutex::new(srv))),
        Err(err) => Err(err.into()),
    }
}

fn run_network(
    net_srv: Arc<Mutex<Libp2pService>>,
    acks: Arc<Mutex<HashMap<u128, Sender<()>>>>,
) -> (
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
    impl Future<Item = (), Error = std::io::Error>,
) {
    let (mut _tx, net_rx) = mpsc::unbounded();
    let (net_tx, mut _rx) = mpsc::unbounded::<NetworkMessage>();
    let net_srv_1 = net_srv.clone();
    let connected_fut = future::poll_fn(move || {
        match try_ready!(net_srv_1.lock().poll()) {
            Some(ServiceEvent::OpenedCustomProtocol { peer_id, .. }) => {
                debug!(
                    "Connected peer: {}",
                    convert_peer_id_to_account_address(&peer_id).unwrap()
                );
            }
            _ => {
                debug!("Connected checked");
            }
        }
        Ok(Async::Ready(()))
    });

    let net_srv_2 = net_srv.clone();
    let ack_sender = net_srv.clone();
    let task_notify = Arc::new(AtomicTask::new());
    let notify = task_notify.clone();
    let network_fut = stream::poll_fn(move || {
        notify.register();
        net_srv_2.lock().poll()
    })
    .for_each(move |event| {
        match event {
            ServiceEvent::CustomMessage { peer_id, message } => {
                //todo: Error handle
                let message = Message::from_bytes(message.as_ref()).unwrap();
                match message {
                    Message::Payload(payload) => {
                        //receive message
                        info!("Receive message with peer_id:{:?}", &peer_id);
                        let address = convert_peer_id_to_account_address(&peer_id).unwrap();
                        let user_msg = NetworkMessage {
                            peer_id: address,
                            data: payload.data,
                        };
                        let _ = _tx.unbounded_send(user_msg);
                        if payload.id != 0 {
                            ack_sender.lock().send_custom_message(
                                &peer_id,
                                Message::ACK(payload.id).into_bytes(),
                            );
                        }
                    }
                    Message::ACK(message_id) => {
                        info!("Receive message ack");
                        if let Some(tx) = acks.lock().remove(&message_id) {
                            let _ = tx.send(());
                        } else {
                            error!(
                                "Receive a invalid ack, message id:{}, peer id:{}",
                                message_id, peer_id
                            );
                        }
                    }
                }
            }
            ServiceEvent::OpenedCustomProtocol {
                peer_id,
                version: _,
                debug_info: _,
            } => {
                info!(
                    "Connected peer {:?}",
                    convert_peer_id_to_account_address(&peer_id).unwrap()
                );
            }
            ServiceEvent::ClosedCustomProtocol {
                peer_id: _,
                debug_info: _,
            } => debug!("Network close custom protol"),
            ServiceEvent::Clogged {
                peer_id: _,
                messages: _,
            } => debug!("Network clogged"),
        };
        Ok(())
    })
    .then(|_| {
        debug!("Finish network poll");
        Ok(())
    });

    let protocol_fut = stream::poll_fn(move || _rx.poll())
        .for_each(move |message| {
            let peer_id = convert_account_address_to_peer_id(message.peer_id).unwrap();
            net_srv
                .lock()
                .send_custom_message(&peer_id, Message::new_message(message.data).into_bytes());
            task_notify.notify();
            if net_srv.lock().is_open(&peer_id) == false {
                error!(
                    "Message send to peer :{} is not connected",
                    convert_peer_id_to_account_address(&peer_id).unwrap()
                );
            }
            info!("Already send message {:?}", &peer_id);
            Ok(())
        })
        .then(|res| {
            match res {
                Ok(()) => {
                    debug!("Finish prototol poll");
                }
                Err(_) => error!("protocol disconnected"),
            };
            Ok(())
        });
    let futures: Vec<Box<dyn Future<Item = (), Error = io::Error> + Send>> = vec![
        Box::new(network_fut) as Box<_>,
        Box::new(protocol_fut) as Box<_>,
    ];

    let futs = futures::select_all(futures)
        .and_then(move |_| {
            debug!("Networking ended");
            Ok(())
        })
        .map_err(|(r, _, _)| r);

    let futs = connected_fut.and_then(move |_| futs);

    (net_tx, net_rx, futs)
}

fn spawn_network(
    libp2p_service: Arc<Mutex<Libp2pService>>,
    acks: Arc<Mutex<HashMap<u128, Sender<()>>>>,
    close_rx: oneshot::Receiver<()>,
) -> (
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
) {
    let (network_sender, network_receiver, network_future) = run_network(libp2p_service, acks);
    let fut = network_future
        .select(close_rx.then(|_| {
            debug!("Shutdown the network");
            Ok(())
        }))
        .map(|(val, _)| val)
        .map_err(|(_err, _)| ());
    let mut runtime = tokio::runtime::Builder::new()
        .name_prefix("libp2p-")
        .build()
        .unwrap();
    let _thread = thread::Builder::new()
        .name("network".to_string())
        .spawn(move || {
            let _ = runtime.block_on(fut);
        });
    (network_sender, network_receiver)
}

impl NetworkService {
    fn new(
        cfg: NetworkConfiguration,
    ) -> (
        NetworkService,
        mpsc::UnboundedSender<NetworkMessage>,
        mpsc::UnboundedReceiver<NetworkMessage>,
        oneshot::Sender<()>,
    ) {
        let (close_tx, close_rx) = oneshot::channel::<()>();
        let libp2p_service = build_libp2p_service(cfg).unwrap();
        let acks = Arc::new(Mutex::new(HashMap::new()));
        let (network_sender, network_receiver) =
            spawn_network(libp2p_service.clone(), acks.clone(), close_rx);
        info!("Network started, connected peers:");
        for p in libp2p_service.lock().connected_peers() {
            info!("peer_id:{}", p);
        }

        (
            Self {
                libp2p_service,
                acks,
            },
            network_sender,
            network_receiver,
            close_tx,
        )
    }

    pub fn is_connected(&self, address: AccountAddress) -> bool {
        self.libp2p_service
            .lock()
            .is_open(&convert_account_address_to_peer_id(address).unwrap())
    }

    pub fn identify(&self) -> AccountAddress {
        convert_peer_id_to_account_address(self.libp2p_service.lock().peer_id()).unwrap()
    }

    pub fn send_message(
        &mut self,
        account_address: AccountAddress,
        message: Vec<u8>,
    ) -> impl Future<Item = (), Error = Canceled> {
        let (tx, rx) = oneshot::channel::<()>();
        let (protocol_msg, message_id) = Message::new_payload(message);
        let peer_id =
            convert_account_address_to_peer_id(account_address).expect("Invalid account address");

        self.libp2p_service
            .lock()
            .send_custom_message(&peer_id, protocol_msg.into_bytes());
        debug!("Send message with ack");
        self.acks.lock().insert(message_id, tx);
        rx
    }

    pub fn broadcast_message(&mut self, message: Vec<u8>) {
        debug!("start send broadcast message");
        let (protocol_msg, message_id) = Message::new_payload(message);

        let message_bytes = protocol_msg.into_bytes();

        let mut peers = HashSet::new();

        for p in self.libp2p_service.lock().connected_peers() {
            debug!("will send message to {}", p);
            peers.insert(p.clone());
        }

        for peer_id in peers {
            self.libp2p_service
                .lock()
                .send_custom_message(&peer_id, message_bytes.clone());
        }
        debug!("finish send broadcast message");
    }

}

pub type NetworkComponent = (
    NetworkService,
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
    oneshot::Sender<()>,
);
