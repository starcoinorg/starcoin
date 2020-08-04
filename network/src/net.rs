// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{NetworkMessage, PeerEvent};

use anyhow::*;
use bytes::Bytes;
use config::NetworkConfig;
use crypto::hash::HashValue;
use futures::{channel::mpsc, prelude::*};
use libp2p::PeerId;
use network_p2p::{
    identity, Event, Multiaddr, NetworkConfiguration, NetworkService, NetworkWorker, NodeKeyConfig,
    Params, Secret, PROTOCOL_NAME,
};
use parity_codec::alloc::collections::HashSet;
use std::borrow::Cow;
use std::sync::Arc;
use types::peer_info::PeerInfo;
use types::PROTOCOLS;

const PROTOCOL_ID: &[u8] = b"starcoin";

#[derive(Clone)]
pub struct SNetworkService {
    inner: NetworkInner,
    service: Arc<NetworkService>,
    net_tx: Option<mpsc::UnboundedSender<NetworkMessage>>,
}

#[derive(Clone)]
pub struct NetworkInner {
    service: Arc<NetworkService>,
}

impl SNetworkService {
    pub fn new(cfg: NetworkConfiguration) -> Self {
        let protocol = network_p2p::ProtocolId::from(PROTOCOL_ID);

        let worker = NetworkWorker::new(Params::new(cfg, protocol)).unwrap();
        let service = worker.service().clone();
        let worker = worker;

        async_std::task::spawn(worker);

        let inner = NetworkInner {
            service: service.clone(),
        };

        Self {
            inner,
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
        let (tx, net_rx) = mpsc::unbounded();
        let (net_tx, rx) = mpsc::unbounded::<NetworkMessage>();
        let (event_tx, event_rx) = mpsc::unbounded::<PeerEvent>();
        let inner = self.inner.clone();

        self.net_tx = Some(net_tx.clone());

        async_std::task::spawn(Self::start_network(inner, tx, rx, event_tx, close_rx));
        (net_tx, net_rx, event_rx, close_tx)
    }

    async fn start_network(
        inner: NetworkInner,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
        net_rx: mpsc::UnboundedReceiver<NetworkMessage>,
        event_tx: mpsc::UnboundedSender<PeerEvent>,
        close_rx: mpsc::UnboundedReceiver<()>,
    ) {
        let mut event_stream = inner.service.event_stream().fuse();
        let mut net_rx = net_rx.fuse();
        let mut close_rx = close_rx.fuse();

        loop {
            futures::select! {
                message = net_rx.select_next_some()=>{
                    inner.handle_network_send(message).await.unwrap();
                },
                event = event_stream.select_next_some()=>{
                    inner.handle_network_receive(event,net_tx.clone(),event_tx.clone()).await.unwrap();
                },
                _ = close_rx.select_next_some() => {
                    //TODO
                    debug!("To shutdown command ");
                    break;
                }
                complete => {
                    debug!("all stream are complete");
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

    pub async fn send_message(
        &self,
        peer_id: PeerId,
        protocol_name: Cow<'static, [u8]>,
        message: Vec<u8>,
    ) -> Result<()> {
        debug!("Send message to {}", &peer_id);
        self.service
            .write_notification(peer_id, protocol_name, message);

        Ok(())
    }

    pub async fn broadcast_message(&mut self, protocol_name: Cow<'static, [u8]>, message: Vec<u8>) {
        debug!("broadcast message, protocol: {:?}", protocol_name);
        self.service.broadcast_message(protocol_name, message).await;
    }

    pub async fn connected_peers(&self) -> HashSet<PeerId> {
        self.service.connected_peers().await
    }

    pub fn update_self_info(&self, info: PeerInfo) {
        self.service.update_self_info(info);
    }

    pub async fn get_address(&self, peer_id: PeerId) -> Vec<Multiaddr> {
        self.service.get_address(peer_id).await
    }

    pub async fn exist_notif_proto(&self, protocol_name: Cow<'static, [u8]>) -> bool {
        self.service.exist_notif_proto(protocol_name).await
    }

    pub async fn sub_stream(&self, protocol_name: Cow<'static, [u8]>) -> impl Stream<Item = Event> {
        self.service.sub_stream(protocol_name)
    }
}

impl NetworkInner {
    async fn handle_network_receive(
        &self,
        event: Event,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
        event_tx: mpsc::UnboundedSender<PeerEvent>,
    ) -> Result<()> {
        match event {
            Event::Dht(_) => {
                debug!("ignore dht event");
            }
            Event::NotificationStreamOpened { remote, info } => {
                debug!(
                    "Connected peer {:?},Myself is {:?}",
                    remote,
                    self.service.peer_id()
                );
                let open_msg = PeerEvent::Open(remote.into(), Box::new(info.as_ref().clone()));
                event_tx.unbounded_send(open_msg)?;
            }
            Event::NotificationStreamClosed { remote } => {
                debug!(
                    "Close peer {:?},Myself is {:?}",
                    remote,
                    self.service.peer_id()
                );
                let open_msg = PeerEvent::Close(remote.into());
                event_tx.unbounded_send(open_msg)?;
            }
            Event::NotificationsReceived {
                remote,
                protocol_name,
                messages,
            } => {
                self.handle_messages(remote, protocol_name, messages, net_tx)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_messages(
        &self,
        peer_id: PeerId,
        protocol_name: Cow<'static, [u8]>,
        messages: Vec<Bytes>,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
    ) -> Result<()> {
        debug!("Receive message with peer_id:{:?}", &peer_id);
        for message in messages {
            //receive message
            let network_msg = NetworkMessage {
                peer_id: peer_id.clone(),
                protocol_name: protocol_name.clone(),
                data: message.to_vec(),
            };
            net_tx.unbounded_send(network_msg)?;
        }
        Ok(())
    }

    async fn handle_network_send(&self, message: NetworkMessage) -> Result<()> {
        let account_addr = message.peer_id.clone();
        self.service
            .write_notification(account_addr, PROTOCOL_NAME.into(), message.data);
        Ok(())
    }
}

pub fn build_network_service(
    cfg: &NetworkConfig,
    genesis_hash: HashValue,
    self_info: PeerInfo,
) -> (
    SNetworkService,
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
    mpsc::UnboundedReceiver<PeerEvent>,
    mpsc::UnboundedSender<()>,
) {
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
        protocols: PROTOCOLS.clone(),
        genesis_hash,
        self_info,
        ..NetworkConfiguration::default()
    };
    let mut service = SNetworkService::new(config);
    let (net_tx, net_rx, event_rx, control_tx) = service.run();
    (service, net_tx, net_rx, event_rx, control_tx)
}
