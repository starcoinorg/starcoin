// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::build_network_worker;
use crate::network::Inner;
use anyhow::{format_err, Result};
use config::NodeConfig;
use futures::future::{abortable, AbortHandle};
use futures::FutureExt;
use lru::LruCache;
use network_api::messages::{
    GetPeerById, GetPeerSet, GetSelfPeer, NotificationMessage, PeerEvent, PeerMessage,
    ReportReputation,
};
use network_api::{NetworkActor, PeerMessageHandler};
use network_p2p::{Event, NetworkConfiguration, NetworkService, NetworkWorker, ProtocolId};
use starcoin_crypto::HashValue;
use starcoin_metrics::Registry;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_types::peer_info::{PeerId, PeerInfo, RpcInfo};
use starcoin_types::startup_info::ChainInfo;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NetworkActorService {
    config: Arc<NodeConfig>,
    worker: Option<NetworkWorker>,
    inner: Inner,
    network_worker_handle: Option<AbortHandle>,
}

impl NetworkActor for NetworkActorService {}

impl NetworkActorService {
    pub fn new<H>(
        config: Arc<NodeConfig>,
        chain_info: ChainInfo,
        rpc: Option<(RpcInfo, ServiceRef<NetworkRpcService>)>,
        peer_message_handler: H,
    ) -> Result<Self>
    where
        H: PeerMessageHandler + 'static,
    {
        let worker = build_network_worker(
            config.as_ref(),
            chain_info.clone(),
            NotificationMessage::protocols(),
            rpc,
        )?;
        let service = worker.service().clone();
        let self_info = PeerInfo::new(config.network.self_peer_id()?, chain_info);
        let inner = Inner::new(self_info, service, peer_message_handler)?;
        Ok(Self {
            config,
            worker: Some(worker),
            inner,
            network_worker_handle: None,
        })
    }

    pub fn network_service(&self) -> Arc<network_p2p::NetworkService> {
        self.inner.network_service.clone()
    }
}

impl ActorService for NetworkActorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let worker = self
            .worker
            .take()
            .ok_or_else(|| format_err!("Network worker should init before started."))?;
        let event_stream = self.inner.network_service.event_stream("network");
        ctx.add_stream(event_stream);
        let (fut, abort_handle) = abortable(worker);
        self.network_worker_handle = Some(abort_handle);
        ctx.spawn(fut.then(|result| async {
            match result {
                Err(_abort) => info!("Network worker stopped."),
                Ok(Err(e)) => error!("Network worker unexpect stopped for : {:?}", e),
                Ok(Ok(_)) => {}
            }
        }));
        Ok(())
    }

    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        if let Some(abort_handle) = self.network_worker_handle.take() {
            abort_handle.abort();
        }
        Ok(())
    }
}

impl EventHandler<Self, Event> for NetworkActorService {
    fn handle_event(&mut self, event: Event, ctx: &mut ServiceContext<NetworkActorService>) {
        match event {
            Event::Dht(_) => {
                debug!("ignore dht event");
            }
            Event::NotificationStreamOpened { remote, info } => {
                debug!("Connected peer {:?}", remote);
                let peer_event = PeerEvent::Open(remote.clone().into(), info.clone());
                self.inner.on_peer_connected(remote.into(), *info);
                ctx.broadcast(peer_event);
            }
            Event::NotificationStreamClosed { remote } => {
                debug!("Close peer {:?}", remote);
                let peer_event = PeerEvent::Close(remote.clone().into());
                self.inner.on_peer_disconnected(remote.into());
                ctx.broadcast(peer_event);
            }
            Event::NotificationsReceived {
                remote,
                protocol,
                messages,
            } => {
                for message in messages {
                    if let Err(e) = self.inner.handle_network_message(
                        remote.clone().into(),
                        protocol.clone(),
                        message,
                    ) {
                        error!(
                            "Handle network message fail, remote:{}, protocol:{}, error: {:?}",
                            remote, protocol, e
                        )
                    }
                }
            }
        }
    }
}

impl EventHandler<Self, ReportReputation> for NetworkActorService {
    fn handle_event(
        &mut self,
        msg: ReportReputation,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) {
        self.inner
            .network_service
            .report_peer(msg.peer_id.into(), msg.change);
    }
}

impl EventHandler<Self, NotificationMessage> for NetworkActorService {
    fn handle_event(
        &mut self,
        msg: NotificationMessage,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) {
        self.inner.broadcast(msg);
    }
}

impl EventHandler<Self, PeerMessage> for NetworkActorService {
    fn handle_event(&mut self, msg: PeerMessage, _ctx: &mut ServiceContext<NetworkActorService>) {
        self.inner.send_peer_message(msg.peer_id, msg.notification);
    }
}

impl ServiceHandler<Self, GetPeerSet> for NetworkActorService {
    fn handle(
        &mut self,
        _msg: GetPeerSet,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <GetPeerSet as ServiceRequest>::Response {
        self.inner
            .peers
            .iter()
            .map(|(_, peer)| peer.get_peer_info().clone())
            .collect::<Vec<_>>()
    }
}

impl ServiceHandler<Self, GetPeerById> for NetworkActorService {
    fn handle(
        &mut self,
        msg: GetPeerById,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <GetPeerById as ServiceRequest>::Response {
        self.inner
            .peers
            .get(&msg.peer_id)
            .map(|peer| peer.get_peer_info().clone())
    }
}

impl ServiceHandler<Self, GetSelfPeer> for NetworkActorService {
    fn handle(
        &mut self,
        _msg: GetSelfPeer,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <GetSelfPeer as ServiceRequest>::Response {
        self.inner.self_peer.get_peer_info().clone()
    }
}
