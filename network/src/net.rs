// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{helper::convert_boot_nodes, PayloadMsg, PeerEvent};
use crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};

use crate::messages::{Message, NetworkMessage};
use anyhow::*;
use futures::{
    channel::{
        mpsc,
        oneshot::{self, Canceled, Sender},
    },
    prelude::*,
};

use anyhow::*;
use bytes::Bytes;
use config::NetworkConfig;
use libp2p::PeerId;
use network_p2p::{
    identity, Event, GenericProtoOut as ServiceEvent, NetworkConfiguration, NetworkService,
    NetworkWorker, NodeKeyConfig, Params, Secret,
};
use parity_codec::alloc::collections::HashSet;
use parking_lot::Mutex;
use scs::SCSCodec;
use slog::Drain;
use std::cell::{Cell, RefCell};
use std::task::{Context, Poll};
use std::{collections::HashMap, io, sync::Arc, thread};
use tokio::runtime::Handle;
use types::account_address::AccountAddress;

#[derive(Clone)]
pub struct SNetworkService {
    handle: Handle,
    inner: NetworkInner,
    service: Arc<NetworkService>,
    net_tx: Option<mpsc::UnboundedSender<NetworkMessage>>,
}

#[derive(Clone)]
pub struct NetworkInner {
    service: Arc<NetworkService>,
    acks: Arc<Mutex<HashMap<u128, Sender<()>>>>,
}

impl SNetworkService {
    pub fn new(cfg: NetworkConfiguration, handle: Handle) -> Self {
        let protocol = network_p2p::ProtocolId::from("stargate".as_bytes());

        let worker = NetworkWorker::new(Params::new(cfg, protocol)).unwrap();
        let service = worker.service().clone();
        let worker = worker;

        let acks = Arc::new(Mutex::new(HashMap::new()));

        handle.spawn(worker);

        let inner = NetworkInner {
            service: service.clone(),
            acks,
        };

        Self {
            inner,
            handle,
            service,
            net_tx: None,
        }
    }

    pub fn run(
        &mut self,
    ) -> (
        mpsc::UnboundedSender<NetworkMessage>,
        mpsc::UnboundedReceiver<NetworkMessage>,
        mpsc::UnboundedReceiver<PeerEvent>,
        mpsc::UnboundedSender<()>,
    ) {
        let (close_tx, close_rx) = mpsc::unbounded::<()>();
        let (mut tx, net_rx) = mpsc::unbounded();
        let (net_tx, mut rx) = mpsc::unbounded::<NetworkMessage>();
        let (event_tx, mut event_rx) = mpsc::unbounded::<PeerEvent>();
        let inner = self.inner.clone();

        self.net_tx = Some(net_tx.clone());

        self.handle
            .spawn(Self::start_network(inner, tx, rx, event_tx, close_rx));
        (net_tx, net_rx, event_rx, close_tx)
    }

    async fn start_network(
        inner: NetworkInner,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
        mut net_rx: mpsc::UnboundedReceiver<NetworkMessage>,
        event_tx: mpsc::UnboundedSender<PeerEvent>,
        mut close_rx: mpsc::UnboundedReceiver<()>,
    ) {
        let mut event_stream = inner.service.event_stream().fuse();
        let mut net_rx = net_rx.fuse();
        let mut close_rx = close_rx.fuse();

        loop {
            futures::select! {
                message = net_rx.select_next_some()=>{
                    inner.handle_network_send(message).await.unwrap();
                    info!("send net message");
                },
                event = event_stream.select_next_some()=>{
                    inner.handle_network_receive(event,net_tx.clone(),event_tx.clone()).await.unwrap();
                    info!("receive net message");
                },
                _ = close_rx.select_next_some() => {
                    debug!("To shutdown command ");
                    break;
                }
                complete => {
                    warn!("all stream are complete");
                    break;
                }
            }
        }
    }

    pub async fn is_connected(&self, peer_id: PeerId) -> Result<bool> {
        Ok(self.service.is_connected(peer_id).await)
    }

    pub fn identify(&self) -> &PeerId {
        self.service.peer_id()
    }

    pub async fn send_message(&self, peer_id: PeerId, message: Vec<u8>) -> Result<()> {
        let (tx, rx) = oneshot::channel::<()>();
        let (protocol_msg, message_id) = Message::new_payload(message);

        info!("send message is  {:?}", protocol_msg);
        self.service
            .send_notification(peer_id, protocol_msg.into_bytes());
        info!("Send message with ack");
        //self.waker.wake();
        self.inner.acks.lock().insert(message_id, tx);
        rx.await?;

        Ok(())
    }

    pub async fn broadcast_message(&mut self, message: Vec<u8>) {
        info!("broadcast message");
        let (protocol_msg, message_id) = Message::new_payload(message);

        let message_bytes = protocol_msg.into_bytes();

        self.service.broadcast_message(message_bytes).await;
    }

    pub async fn connected_peers(&self) -> HashSet<PeerId> {
        self.service.connected_peers().await
    }
}

impl NetworkInner {
    async fn handle_network_receive(
        &self,
        event: Event,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
        event_tx: mpsc::UnboundedSender<PeerEvent>,
    ) -> Result<()> {
        info!("message is {:?}", event);
        match event {
            Event::Dht(_) => {
                info!("ignore dht event");
            }
            Event::NotificationStreamOpened { remote } => {
                info!("Connected peer {:?}", remote);
                let open_msg = PeerEvent::Open(remote.into());
                event_tx.unbounded_send(open_msg)?;
            }
            Event::NotificationStreamClosed { remote } => {
                info!("Close peer {:?}", remote);
                let open_msg = PeerEvent::Close(remote.into());
                event_tx.unbounded_send(open_msg)?;
            }
            Event::NotificationsReceived { remote, messages } => {
                self.handle_messages(remote, messages, net_tx).await?;
            }
        }
        Ok(())
    }

    async fn handle_messages(
        &self,
        peer_id: PeerId,
        messages: Vec<Bytes>,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
    ) -> Result<()> {
        info!("Receive message with peer_id:{:?}", &peer_id);
        for message in messages {
            let message = Message::from_bytes(message.as_ref())?;
            info!("message is {:?}", message);
            match message {
                Message::Payload(payload) => {
                    //receive message
                    let user_msg = NetworkMessage {
                        peer_id: peer_id.clone().into(),
                        data: payload.data,
                    };
                    net_tx.unbounded_send(user_msg)?;
                    if payload.id != 0 {
                        self.service.send_notification(
                            peer_id.clone(),
                            Message::ACK(payload.id).into_bytes(),
                        );
                    }
                }
                Message::ACK(message_id) => {
                    info!("Receive message ack");
                    if let Some(mut tx) = self.acks.lock().remove(&message_id) {
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
        Ok(())
    }

    async fn handle_network_send(&self, message: NetworkMessage) -> Result<()> {
        let account_addr = message.peer_id.clone();
        self.service.send_notification(
            account_addr.into(),
            Message::new_payload(message.data).0.into_bytes(),
        );
        Ok(())
    }
}

pub fn build_network_service(
    cfg: &NetworkConfig,
    handle: Handle,
) -> (
    SNetworkService,
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
    mpsc::UnboundedReceiver<PeerEvent>,
    mpsc::UnboundedSender<()>,
) {
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
    let mut service = SNetworkService::new(config, handle);
    let (net_tx, net_rx, event_rx, control_tx) = service.run();
    (service, net_tx, net_rx, event_rx, control_tx)
}
