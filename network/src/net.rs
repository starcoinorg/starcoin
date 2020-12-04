// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{NetworkMessage, PeerEvent};
use anyhow::*;
use bitflags::_core::time::Duration;
use bytes::Bytes;
use config::NetworkConfig;
use futures::channel::mpsc::channel;
use futures::{channel::mpsc, prelude::*};
use network_p2p::config::{RequestResponseConfig, TransportConfig};
use network_p2p::{
    identity, Event, Multiaddr, NetworkConfiguration, NetworkService, NetworkWorker, NodeKeyConfig,
    Params, ProtocolId, Secret,
};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::{is_memory_addr, PeerId, ProtocolRequest, RequestFailure};
use prometheus::{default_registry, Registry};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::ServiceRef;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;
use types::peer_info::RpcInfo;
use types::startup_info::{ChainInfo, ChainStatus};

#[derive(Clone)]
pub struct SNetworkService {
    protocol: ProtocolId,
    chain_info: ChainInfo,
    cfg: NetworkConfiguration,
    metrics_registry: Option<Registry>,
    service: Option<Arc<NetworkService>>,
    net_tx: Option<mpsc::UnboundedSender<NetworkMessage>>,
}

#[derive(Clone)]
pub struct NetworkInner {
    service: Arc<NetworkService>,
}

impl SNetworkService {
    pub fn new(
        protocol: ProtocolId,
        chain_info: ChainInfo,
        cfg: NetworkConfiguration,
        metrics_registry: Option<Registry>,
    ) -> Self {
        Self {
            protocol,
            chain_info,
            cfg,
            metrics_registry,
            service: None,
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

        let worker = NetworkWorker::new(Params::new(
            self.cfg.clone(),
            self.protocol.clone(),
            self.chain_info.clone(),
            self.metrics_registry.clone(),
        ))
        .unwrap();
        self.net_tx = Some(net_tx.clone());
        self.service = Some(worker.service().clone());
        async_std::task::spawn(Self::start_network(worker, tx, rx, event_tx, close_rx));
        (net_tx, net_rx, event_rx, close_tx)
    }

    async fn start_network(
        mut worker: NetworkWorker,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
        net_rx: mpsc::UnboundedReceiver<NetworkMessage>,
        event_tx: mpsc::UnboundedSender<PeerEvent>,
        close_rx: mpsc::UnboundedReceiver<()>,
    ) {
        let inner = NetworkInner::new(worker.service().clone());
        let mut event_stream = inner.service.event_stream("network").fuse();
        let mut net_rx = net_rx.fuse();
        let mut close_rx = close_rx.fuse();

        loop {
            futures::select! {
                message = net_rx.select_next_some()=>{
                    inner.handle_network_send(message).await;
                },
                event = event_stream.select_next_some()=>{
                    inner.handle_network_receive(event,net_tx.clone(),event_tx.clone()).await;
                },
                _ = close_rx.select_next_some() => {
                    info!("Network shutdown");
                    break;
                }
                _ = (&mut worker).fuse() => {},
                complete => {
                    debug!("all stream are complete");
                    break;
                }
            }
        }
    }
    fn service(&self) -> &Arc<NetworkService> {
        self.service
            .as_ref()
            .expect("Should call network function after network running.")
    }

    pub async fn is_connected(&self, peer_id: PeerId) -> Result<bool> {
        Ok(self.service().is_connected(peer_id).await)
    }

    pub fn identify(&self) -> &PeerId {
        self.service().peer_id()
    }

    pub async fn send_message(
        &self,
        peer_id: PeerId,
        protocol_name: Cow<'static, str>,
        message: Vec<u8>,
    ) -> Result<()> {
        debug!("Send message to {}", &peer_id);
        self.service()
            .write_notification(peer_id, protocol_name, message);

        Ok(())
    }

    pub async fn request(
        &self,
        target: network_api::PeerId,
        protocol: impl Into<Cow<'static, str>>,
        request: Vec<u8>,
    ) -> Result<Vec<u8>, RequestFailure> {
        let protocol = protocol.into();
        debug!("Send request to peer {} and rpc: {:?}", target, protocol);
        self.service()
            .request(target.into(), protocol, request)
            .await
    }

    pub async fn broadcast_message(&mut self, protocol_name: Cow<'static, str>, message: Vec<u8>) {
        debug!("broadcast message, protocol: {:?}", protocol_name);
        self.service()
            .broadcast_message(protocol_name, message)
            .await;
    }

    pub fn add_peer(&self, peer: String) -> Result<()> {
        self.service()
            .add_reserved_peer(peer)
            .map_err(|e| format_err!("{:?}", e))
    }

    pub async fn known_peers(&self) -> HashSet<PeerId> {
        self.service().known_peers().await
    }

    pub async fn network_state(&self) -> Result<NetworkState> {
        self.service()
            .network_state()
            .await
            .map_err(|_| format_err!("request cancel."))
    }

    pub fn update_chain_status(&self, chain_status: ChainStatus) {
        self.service().update_chain_status(chain_status);
    }

    pub async fn get_address(&self, peer_id: PeerId) -> Vec<Multiaddr> {
        self.service().get_address(peer_id).await
    }

    pub async fn exist_notif_proto(&self, protocol_name: Cow<'static, str>) -> bool {
        self.service().exist_notif_proto(protocol_name).await
    }
}

impl NetworkInner {
    pub fn new(service: Arc<NetworkService>) -> Self {
        Self { service }
    }
    pub async fn handle_network_receive(
        &self,
        event: Event,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
        event_tx: mpsc::UnboundedSender<PeerEvent>,
    ) {
        if let Err(e) = self
            .handle_network_receive_inner(event, net_tx, event_tx)
            .await
        {
            error!("handle_network_receive error: {:?}", e);
        }
    }

    pub(crate) async fn handle_network_receive_inner(
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
                debug!("Connected peer {:?}", remote);
                let open_msg = PeerEvent::Open(remote.into(), info);
                event_tx.unbounded_send(open_msg)?;
            }
            Event::NotificationStreamClosed { remote } => {
                debug!("Close peer {:?}", remote);
                let open_msg = PeerEvent::Close(remote.into());
                event_tx.unbounded_send(open_msg)?;
            }
            Event::NotificationsReceived {
                remote,
                protocol,
                messages,
            } => {
                self.handle_messages(remote, protocol, messages, net_tx)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_messages(
        &self,
        peer_id: PeerId,
        protocol: Cow<'static, str>,
        messages: Vec<Bytes>,
        net_tx: mpsc::UnboundedSender<NetworkMessage>,
    ) -> Result<()> {
        debug!(
            "Receive message with peer_id:{:?}, protocol: {}",
            &peer_id, protocol
        );
        for message in messages {
            //receive message
            let network_msg = NetworkMessage {
                peer_id: peer_id.clone(),
                protocol_name: protocol.clone(),
                data: message.to_vec(),
            };
            net_tx.unbounded_send(network_msg)?;
        }
        Ok(())
    }

    async fn handle_network_send(&self, message: NetworkMessage) {
        let peer_id = message.peer_id.clone();
        self.service
            .write_notification(peer_id, message.protocol_name, message.data);
    }
}

const MAX_REQUEST_SIZE: u64 = 1024 * 1024;
const MAX_RESPONSE_SIZE: u64 = 1024 * 1024 * 64;
const REQUEST_BUFFER_SIZE: usize = 128;
pub const RPC_PROTOCOL_PREFIX: &str = "/starcoin/rpc/";

pub fn build_network_service(
    node_name: String,
    chain_info: ChainInfo,
    cfg: &NetworkConfig,
    protocols: Vec<Cow<'static, str>>,
    rpc_service: Option<(RpcInfo, ServiceRef<NetworkRpcService>)>,
) -> (
    SNetworkService,
    mpsc::UnboundedSender<NetworkMessage>,
    mpsc::UnboundedReceiver<NetworkMessage>,
    mpsc::UnboundedReceiver<PeerEvent>,
    mpsc::UnboundedSender<()>,
) {
    let transport_config = if is_memory_addr(&cfg.listen) {
        TransportConfig::MemoryOnly
    } else {
        TransportConfig::Normal {
            //TODO support enable mdns by config.
            enable_mdns: false,
            allow_private_ipv4: false,
            wasm_external_transport: None,
        }
    };
    //let rpc_info: Vec<String> = starcoin_network_rpc_api::gen_client::get_rpc_info();
    //TODO define RequestResponseConfig by rpc api
    let rpc_protocols = match rpc_service {
        Some((rpc_info, rpc_service)) => rpc_info
            .into_iter()
            .map(|rpc_path| {
                //TODO define rpc path in rpc api, and add prefix.
                let protocol_name: Cow<'static, str> =
                    format!("{}{}", RPC_PROTOCOL_PREFIX, rpc_path.as_str()).into();
                let rpc_path_for_stream: Cow<'static, str> = rpc_path.into();
                let (sender, receiver) = channel(REQUEST_BUFFER_SIZE);
                let stream = receiver.map(move |request| ProtocolRequest {
                    protocol: rpc_path_for_stream.clone(),
                    request,
                });
                if let Err(e) = rpc_service.add_event_stream(stream) {
                    error!(
                        "Add request event stream for rpc {} fail: {:?}",
                        protocol_name, e
                    );
                }
                RequestResponseConfig {
                    name: protocol_name,
                    max_request_size: MAX_REQUEST_SIZE,
                    max_response_size: MAX_RESPONSE_SIZE,
                    request_timeout: Duration::from_secs(15),
                    inbound_queue: Some(sender),
                }
            })
            .collect::<Vec<_>>(),
        None => vec![],
    };
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
        protocols,
        request_response_protocols: rpc_protocols,
        transport: transport_config,
        node_name,
        client_version: config::APP_NAME_WITH_VERSION.clone(),
        ..NetworkConfiguration::default()
    };
    // protocol id is chain/{chain_id}, `RegisteredProtocol` will append `/starcoin` prefix
    let protocol_id = ProtocolId::from(format!("chain/{}", chain_info.chain_id()).as_str());

    let mut service = SNetworkService::new(
        protocol_id,
        chain_info,
        config,
        //TODO use a custom registry for each instance.
        Some(default_registry().clone()),
    );
    let (net_tx, net_rx, event_rx, control_tx) = service.run();
    (service, net_tx, net_rx, event_rx, control_tx)
}
