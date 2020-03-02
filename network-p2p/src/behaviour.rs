// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::custom_proto::{CustomProto, CustomProtoOut};
use crate::debug_info::DebugInfoBehaviour;
use crate::discovery::{DiscoveryBehaviour, DiscoveryOut};
use crate::DiscoveryNetBehaviour;
use crate::{debug_info, ProtocolId};
use bytes::BytesMut;
use futures::prelude::*;
use log::debug;

use libp2p::{
    core::{ConnectedPoint, Multiaddr, PeerId, PublicKey},
    identify::IdentifyInfo,
    kad::record::Key,
    swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess},
    NetworkBehaviour,
};
use std::task::Poll;
use std::{borrow::Cow, iter, time::Duration};
use void;

/// General behaviour of the network.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourOut", poll_method = "poll")]
pub struct Behaviour<TSubstream> {
    /// Custom protocols (dot, bbq, sub, etc.).
    custom_protocols: CustomProto<TSubstream>,
    /// Discovers nodes of the network. Defined below.
    discovery: DiscoveryBehaviour,
    debug_info: DebugInfoBehaviour,
    /// Queue of events to produce for the outside.
    #[behaviour(ignore)]
    events: Vec<BehaviourOut>,
}

impl<TSubstream> Behaviour<TSubstream> {
    /// Builds a new `Behaviour`.
    pub fn new(
        protocol: impl Into<ProtocolId>,
        user_agent: String,
        local_public_key: PublicKey,
        known_addresses: Vec<(PeerId, Multiaddr)>,
        enable_mdns: bool,
        allow_private_ipv4: bool,
        peerset: peerset::Peerset,
        discovery_only_if_under_num: u64,
    ) -> Self {
        //todo: set versions
        let versions = "/stargate/0.1".as_bytes();
        //todo: set protocol id
        let custom_protocols = CustomProto::new(protocol, &versions, peerset);
        Behaviour {
            custom_protocols,
            debug_info: DebugInfoBehaviour::new(user_agent, local_public_key.clone()),
            discovery: DiscoveryBehaviour::new(
                local_public_key,
                known_addresses,
                enable_mdns,
                allow_private_ipv4,
                discovery_only_if_under_num,
            ),
            events: Vec::new(),
        }
    }

    /// Sends a message to a peer.
    ///
    /// Has no effect if the custom protocol is not open with the given peer.
    ///
    /// Also note that even we have a valid open substream, it may in fact be already closed
    /// without us knowing, in which case the packet will not be received.
    #[inline]
    pub fn send_custom_message(&mut self, target: &PeerId, data: Vec<u8>) {
        self.custom_protocols.send_packet(target, data)
    }

    /// Returns the list of nodes that we know exist in the network.
    pub fn known_peers(&mut self) -> impl Iterator<Item = &PeerId> {
        self.discovery.known_peers()
    }

    /// Returns true if we try to open protocols with the given peer.
    pub fn is_enabled(&self, peer_id: &PeerId) -> bool {
        self.custom_protocols.is_enabled(peer_id)
    }

    /// Returns true if we have an open protocol with the given peer.
    pub fn is_open(&self, peer_id: &PeerId) -> bool {
        self.custom_protocols.is_open(peer_id)
    }

    /// Adds a hard-coded address for the given peer, that never expires.
    pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.discovery.add_known_address(peer_id, addr)
    }

    /// Disconnects the custom protocols from a peer.
    ///
    /// The peer will still be able to use Kademlia or other protocols, but will get disconnected
    /// after a few seconds of inactivity.
    ///
    /// This is asynchronous and does not instantly close the custom protocols.
    /// Corresponding closing events will be generated once the closing actually happens.
    ///
    /// Has no effect if we're not connected to the `PeerId`.
    #[inline]
    pub fn drop_node(&mut self, peer_id: &PeerId) {
        self.custom_protocols.disconnect_peer(peer_id)
    }

    /// Returns the state of the peerset manager, for debugging purposes.
    pub fn peerset_debug_info(&mut self) -> serde_json::Value {
        self.custom_protocols.peerset_debug_info()
    }
}

/// Event that can be emitted by the behaviour.
#[derive(Debug)]
pub enum BehaviourOut {
    /// Opened a custom protocol with the remote.
    CustomProtocolOpen {
        /// Version of the protocol that has been opened.
        version: u8,
        /// Id of the node we have opened a connection with.
        peer_id: PeerId,
        /// Endpoint used for this custom protocol.
        endpoint: ConnectedPoint,
    },

    /// Closed a custom protocol with the remote.
    CustomProtocolClosed {
        /// Id of the peer we were connected to.
        peer_id: PeerId,
        /// Reason why the substream closed, for diagnostic purposes.
        reason: Cow<'static, str>,
    },

    /// Receives a message on a custom protocol substream.
    CustomMessage {
        /// Id of the peer the message came from.
        peer_id: PeerId,
        /// Message that has been received.
        message: BytesMut,
    },

    /// A substream with a remote is clogged. We should avoid sending more data to it if possible.
    Clogged {
        /// Id of the peer the message came from.
        peer_id: PeerId,
        /// Copy of the messages that are within the buffer, for further diagnostic.
        messages: Vec<Vec<u8>>,
    },

    /// We have obtained debug information from a peer.
    Identified {
        /// Id of the peer that has been identified.
        peer_id: PeerId,
        /// Information about the peer.
        info: IdentifyInfo,
    },

    /// We have successfully pinged a peer.
    PingSuccess {
        /// Id of the peer that has been pinged.
        peer_id: PeerId,
        /// Time it took for the ping to come back.
        ping_time: Duration,
    },
    Event(Event),
}
/// Events generated by DHT as a response to get_value and put_value requests.
#[derive(Debug, Clone)]
#[must_use]
pub enum DhtEvent {
    /// The value was found.
    ValueFound(Vec<(Key, Vec<u8>)>),

    /// The requested record has not been found in the DHT.
    ValueNotFound(Key),

    /// The record has been successfully inserted into the DHT.
    ValuePut(Key),

    /// An error has occured while putting a record into the DHT.
    ValuePutFailed(Key),
}

/// Type for events generated by networking layer.
#[derive(Debug, Clone)]
#[must_use]
pub enum Event {
    /// Event generated by a DHT.
    Dht(DhtEvent),
}

impl From<CustomProtoOut> for BehaviourOut {
    fn from(other: CustomProtoOut) -> BehaviourOut {
        match other {
            CustomProtoOut::CustomProtocolOpen {
                version,
                peer_id,
                endpoint,
            } => BehaviourOut::CustomProtocolOpen {
                version,
                peer_id,
                endpoint,
            },
            CustomProtoOut::CustomProtocolClosed { peer_id, reason } => {
                BehaviourOut::CustomProtocolClosed { peer_id, reason }
            }
            CustomProtoOut::CustomMessage { peer_id, message } => {
                BehaviourOut::CustomMessage { peer_id, message }
            }
            CustomProtoOut::Clogged { peer_id, messages } => {
                BehaviourOut::Clogged { peer_id, messages }
            }
        }
    }
}

impl<TSubstream> NetworkBehaviourEventProcess<void::Void> for Behaviour<TSubstream> {
    fn inject_event(&mut self, event: void::Void) {
        void::unreachable(event)
    }
}

impl<TSubstream> NetworkBehaviourEventProcess<CustomProtoOut> for Behaviour<TSubstream> {
    fn inject_event(&mut self, event: CustomProtoOut) {
        self.events.push(event.into());
    }
}

impl<TSubstream> Behaviour<TSubstream> {
    pub fn poll<TEv>(&mut self) -> Poll<NetworkBehaviourAction<TEv, BehaviourOut>> {
        if !self.events.is_empty() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(self.events.remove(0)));
        }

        Poll::Pending
    }
}

impl<TSubstream> NetworkBehaviourEventProcess<debug_info::DebugInfoEvent>
    for Behaviour<TSubstream>
{
    fn inject_event(&mut self, event: debug_info::DebugInfoEvent) {
        let debug_info::DebugInfoEvent::Identified { peer_id, mut info } = event;
        if info.listen_addrs.len() > 30 {
            debug!(target: "sg-libp2p", "Node {:?} has reported more than 30 addresses; \
                it is identified by {:?} and {:?}", peer_id, info.protocol_version,
                   info.agent_version
            );
            info.listen_addrs.truncate(30);
        }
        for addr in &info.listen_addrs {
            self.discovery
                .add_self_reported_address(&peer_id, addr.clone());
        }
        self.custom_protocols
            .add_discovered_nodes(iter::once(peer_id.clone()));
    }
}

impl<TSubstream> NetworkBehaviourEventProcess<DiscoveryOut> for Behaviour<TSubstream> {
    fn inject_event(&mut self, out: DiscoveryOut) {
        match out {
            DiscoveryOut::UnroutablePeer(_peer_id) => {
                // Obtaining and reporting listen addresses for unroutable peers back
                // to Kademlia is handled by the `Identify` protocol, part of the
                // `DebugInfoBehaviour`. See the `NetworkBehaviourEventProcess`
                // implementation for `DebugInfoEvent`.
            }
            DiscoveryOut::Discovered(peer_id) => {
                self.custom_protocols
                    .add_discovered_nodes(iter::once(peer_id));
            }
            DiscoveryOut::ValueFound(results) => {
                self.events
                    .push(BehaviourOut::Event(Event::Dht(DhtEvent::ValueFound(
                        results,
                    ))));
            }
            DiscoveryOut::ValueNotFound(key) => {
                self.events
                    .push(BehaviourOut::Event(Event::Dht(DhtEvent::ValueNotFound(
                        key,
                    ))));
            }
            DiscoveryOut::ValuePut(key) => {
                self.events
                    .push(BehaviourOut::Event(Event::Dht(DhtEvent::ValuePut(key))));
            }
            DiscoveryOut::ValuePutFailed(key) => {
                self.events
                    .push(BehaviourOut::Event(Event::Dht(DhtEvent::ValuePutFailed(
                        key,
                    ))));
            }
        }
    }
}
