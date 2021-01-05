// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use crate::discovery::DiscoveryConfig;
use crate::protocol::generic_proto::NotificationsSink;
use crate::protocol::{CustomMessageOutcome, Protocol};
use crate::request_responses::{RequestFailure, RequestId, ResponseFailure, SendRequestError};
use crate::{
    discovery::DiscoveryBehaviour, discovery::DiscoveryOut, peer_info, protocol::event::DhtEvent,
    request_responses, DiscoveryNetBehaviour, ProtocolId,
};
use bytes::Bytes;
use libp2p::core::{Multiaddr, PeerId, PublicKey};
use libp2p::identify::IdentifyInfo;
use libp2p::kad::record;
use libp2p::swarm::{
    NetworkBehaviour, NetworkBehaviourAction, NetworkBehaviourEventProcess, PollParameters,
};
use libp2p::NetworkBehaviour;
use log::debug;
use starcoin_types::startup_info::ChainInfo;
use std::borrow::Cow;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;
use std::{iter, task::Context, task::Poll};

/// General behaviour of the network. Combines all protocols together.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourOut", poll_method = "poll")]
pub struct Behaviour {
    protocol: Protocol,
    /// Periodically pings and identifies the nodes we are connected to, and store information in a
    /// cache.
    peer_info: peer_info::PeerInfoBehaviour,
    /// Discovers nodes of the network.
    discovery: DiscoveryBehaviour,
    /// Generic request-reponse protocols.
    request_responses: request_responses::RequestResponsesBehaviour,
    /// Queue of events to produce for the outside.
    #[behaviour(ignore)]
    events: VecDeque<BehaviourOut>,
}

/// Event generated by `Behaviour`.
#[derive(Debug)]
pub enum BehaviourOut {
    /// Opened a substream with the given node with the given notifications protocol.
    ///
    /// The protocol is always one of the notification protocols that have been registered.
    NotificationStreamOpened {
        /// Node we opened the substream with.
        remote: PeerId,
        /// Object that permits sending notifications to the peer.
        notifications_sink: NotificationsSink,
        info: Box<ChainInfo>,
    },

    /// The [`NotificationsSink`] object used to send notifications with the given peer must be
    /// replaced with a new one.
    ///
    /// This event is typically emitted when a transport-level connection is closed and we fall
    /// back to a secondary connection.
    NotificationStreamReplaced {
        /// Id of the peer we are connected to.
        remote: PeerId,
        /// Replacement for the previous [`NotificationsSink`].
        notifications_sink: NotificationsSink,
    },

    /// Closed a substream with the given node. Always matches a corresponding previous
    /// `NotificationStreamOpened` message.
    NotificationStreamClosed {
        /// Node we closed the substream with.
        remote: PeerId,
    },

    /// Messages have been received on one or more notifications protocols.
    NotificationsReceived {
        remote: PeerId,
        protocol: Cow<'static, str>,
        messages: Vec<Bytes>,
    },

    /// We have received a request from a peer and answered it.
    ///
    /// This event is generated for statistics purposes.
    InboundRequest {
        /// Peer which sent us a request.
        peer: PeerId,
        /// Protocol name of the request.
        protocol: Cow<'static, str>,
        /// If `Ok`, contains the time elapsed between when we received the request and when we
        /// sent back the response. If `Err`, the error that happened.
        result: Result<Option<Duration>, ResponseFailure>,
    },

    /// A request initiated using [`Behaviour::send_request`] has succeeded or failed.
    RequestFinished {
        /// Request that has succeeded.
        request_id: RequestId,
        /// Response sent by the remote or reason for failure.
        result: Result<Vec<u8>, RequestFailure>,
    },

    RandomKademliaStarted(ProtocolId),
    /// Events generated by a DHT as a response to get_value or put_value requests as well as the
    /// request duration.
    Dht(DhtEvent, Duration),
}

#[derive(Debug, Clone)]
pub struct RpcRequest {
    remote: PeerId,
    data: Vec<u8>,
}

impl Behaviour {
    /// Builds a new `Behaviour`.
    pub fn new(
        protocol: Protocol,
        user_agent: String,
        local_public_key: PublicKey,
        disco_config: DiscoveryConfig,
        request_response_protocols: Vec<request_responses::ProtocolConfig>,
    ) -> Result<Self, request_responses::RegisterError> {
        Ok(Behaviour {
            protocol,
            // debug_info: debug_info::DebugInfoBehaviour::new(user_agent, local_public_key),
            peer_info: peer_info::PeerInfoBehaviour::new(user_agent, local_public_key),
            discovery: disco_config.finish(),
            request_responses: request_responses::RequestResponsesBehaviour::new(
                request_response_protocols.into_iter(),
            )?,
            events: VecDeque::new(),
        })
    }

    /// Returns the list of nodes that we know exist in the network.
    pub fn known_peers(&mut self) -> HashSet<PeerId> {
        self.discovery.known_peers()
    }

    /// Adds a hard-coded address for the given peer, that never expires.
    pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.discovery.add_known_address(peer_id, addr)
    }

    /// Returns the number of nodes in each Kademlia kbucket for each Kademlia instance.
    ///
    /// Identifies Kademlia instances by their [`ProtocolId`] and kbuckets by the base 2 logarithm
    /// of their lower bound.
    pub fn num_entries_per_kbucket(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (&ProtocolId, Vec<(u32, usize)>)> {
        self.discovery.num_entries_per_kbucket()
    }

    /// Returns the number of records in the Kademlia record stores.
    pub fn num_kademlia_records(&mut self) -> impl ExactSizeIterator<Item = (&ProtocolId, usize)> {
        self.discovery.num_kademlia_records()
    }

    /// Returns the total size in bytes of all the records in the Kademlia record stores.
    pub fn kademlia_records_total_size(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (&ProtocolId, usize)> {
        self.discovery.kademlia_records_total_size()
    }

    pub fn get_address(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.discovery.addresses_of_peer(peer_id)
    }

    /// Returns true if we have a channel open with this node.
    pub fn is_open(&self, peer_id: &PeerId) -> bool {
        self.protocol.is_open(peer_id)
    }

    /// Borrows `self` and returns a struct giving access to the information about a node.
    ///
    /// Returns `None` if we don't know anything about this node. Always returns `Some` for nodes
    /// we're connected to, meaning that if `None` is returned then we're not connected to that
    /// node.
    pub fn node(&self, peer_id: &PeerId) -> Option<peer_info::Node> {
        self.peer_info.node(peer_id)
    }

    /// Start querying a record from the DHT. Will later produce either a `ValueFound` or a `ValueNotFound` event.
    pub fn get_value(&mut self, key: &record::Key) {
        self.discovery.get_value(key);
    }

    /// Starts putting a record into DHT. Will later produce either a `ValuePut` or a `ValuePutFailed` event.
    pub fn put_value(&mut self, key: record::Key, value: Vec<u8>) {
        self.discovery.put_value(key, value);
    }

    /// Initiates sending a request.
    ///
    /// An error is returned if we are not connected to the target peer of if the protocol doesn't
    /// match one that has been registered.
    pub fn send_request(
        &mut self,
        target: &PeerId,
        protocol: &str,
        request: Vec<u8>,
    ) -> Result<RequestId, SendRequestError> {
        self.request_responses
            .send_request(target, protocol, request)
    }

    /// Registers a new notifications protocol.
    ///
    /// Please call `event_stream` before registering a protocol, otherwise you may miss events
    /// about the protocol that you have registered.
    ///
    /// You are very strongly encouraged to call this method very early on. Any connection open
    /// will retain the protocols that were registered then, and not any new one.
    pub fn register_notifications_protocol(&mut self, protocol_name: impl Into<Cow<'static, str>>) {
        let protocol = protocol_name.into();
        let list = self
            .protocol
            .register_notifications_protocol(protocol.clone());

        for (remote, notifications_sink, info) in list {
            //let role = reported_roles_to_observed_role(&self.role, remote, roles);
            self.events
                .push_back(BehaviourOut::NotificationStreamOpened {
                    remote: remote.clone(),
                    notifications_sink: notifications_sink.clone(),
                    info: Box::new(info.clone()),
                });
        }
    }

    /// Returns a shared reference to the user protocol.
    pub fn user_protocol(&self) -> &Protocol {
        &self.protocol
    }

    /// Returns a mutable reference to the user protocol.
    pub fn user_protocol_mut(&mut self) -> &mut Protocol {
        &mut self.protocol
    }
}

impl NetworkBehaviourEventProcess<void::Void> for Behaviour {
    fn inject_event(&mut self, event: void::Void) {
        void::unreachable(event)
    }
}

impl NetworkBehaviourEventProcess<CustomMessageOutcome> for Behaviour {
    fn inject_event(&mut self, event: CustomMessageOutcome) {
        match event {
            CustomMessageOutcome::NotificationStreamOpened {
                remote,
                notifications_sink,
                info,
            } => {
                self.events
                    .push_back(BehaviourOut::NotificationStreamOpened {
                        remote,
                        notifications_sink,
                        info,
                    });
            }
            CustomMessageOutcome::NotificationStreamClosed { remote } => {
                self.events
                    .push_back(BehaviourOut::NotificationStreamClosed { remote });
            }
            CustomMessageOutcome::NotificationsReceived {
                remote,
                protocol,
                messages,
            } => {
                self.events.push_back(BehaviourOut::NotificationsReceived {
                    remote,
                    protocol,
                    messages,
                });
            }
            CustomMessageOutcome::None => {}
            CustomMessageOutcome::NotificationStreamReplaced {
                remote,
                notifications_sink,
            } => {
                self.events
                    .push_back(BehaviourOut::NotificationStreamReplaced {
                        remote,
                        notifications_sink,
                    });
            }
        }
    }
}

impl NetworkBehaviourEventProcess<request_responses::Event> for Behaviour {
    fn inject_event(&mut self, event: request_responses::Event) {
        match event {
            request_responses::Event::InboundRequest {
                peer,
                protocol,
                result,
            } => {
                self.events.push_back(BehaviourOut::InboundRequest {
                    peer,
                    protocol,
                    result,
                });
            }

            request_responses::Event::RequestFinished { request_id, result } => {
                self.events
                    .push_back(BehaviourOut::RequestFinished { request_id, result });
            }
        }
    }
}

impl NetworkBehaviourEventProcess<peer_info::PeerInfoEvent> for Behaviour {
    fn inject_event(&mut self, event: peer_info::PeerInfoEvent) {
        let peer_info::PeerInfoEvent::Identified {
            peer_id,
            info:
                IdentifyInfo {
                    protocol_version,
                    agent_version,
                    mut listen_addrs,
                    protocols,
                    ..
                },
        } = event;

        if listen_addrs.len() > 30 {
            debug!(
                target: "sub-libp2p",
                "Node {:?} has reported more than 30 addresses; it is identified by {:?} and {:?}",
                peer_id, protocol_version, agent_version
            );
            listen_addrs.truncate(30);
        }

        for addr in listen_addrs {
            self.discovery
                .add_self_reported_address(&peer_id, protocols.iter(), addr);
        }
        self.protocol.add_discovered_nodes(iter::once(peer_id));
    }
}

impl NetworkBehaviourEventProcess<DiscoveryOut> for Behaviour {
    fn inject_event(&mut self, out: DiscoveryOut) {
        match out {
            DiscoveryOut::UnroutablePeer(_peer_id) => {
                // Obtaining and reporting listen addresses for unroutable peers back
                // to Kademlia is handled by the `Identify` protocol, part of the
                // `PeerInfoBehaviour`. See the `NetworkBehaviourEventProcess`
                // implementation for `PeerInfoEvent`.
            }
            DiscoveryOut::Discovered(peer_id) => {
                self.protocol.add_discovered_nodes(iter::once(peer_id));
            }
            DiscoveryOut::ValueFound(results, duration) => {
                self.events
                    .push_back(BehaviourOut::Dht(DhtEvent::ValueFound(results), duration));
            }
            DiscoveryOut::ValueNotFound(key, duration) => {
                self.events
                    .push_back(BehaviourOut::Dht(DhtEvent::ValueNotFound(key), duration));
            }
            DiscoveryOut::ValuePut(key, duration) => {
                self.events
                    .push_back(BehaviourOut::Dht(DhtEvent::ValuePut(key), duration));
            }
            DiscoveryOut::ValuePutFailed(key, duration) => {
                self.events
                    .push_back(BehaviourOut::Dht(DhtEvent::ValuePutFailed(key), duration));
            }
            DiscoveryOut::RandomKademliaStarted(protocols) => {
                for protocol in protocols {
                    self.events
                        .push_back(BehaviourOut::RandomKademliaStarted(protocol));
                }
            }
        }
    }
}

impl Behaviour {
    fn poll<TEv>(
        &mut self,
        _: &mut Context,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<TEv, BehaviourOut>> {
        if let Some(event) = self.events.pop_front() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(event));
        }

        Poll::Pending
    }
}
