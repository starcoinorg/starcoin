// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg(test)]

use crate::protocol::generic_proto::{GenericProto, GenericProtoOut};

use futures::prelude::*;
use libp2p::core::{transport::MemoryTransport, upgrade, Endpoint};
use libp2p::swarm::behaviour::{FromSwarm, NewExternalAddrOfPeer};
use libp2p::swarm::{Config as SwarmConfig, ConnectionId, NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{identity, noise, yamux};
use libp2p::{Multiaddr, Transport};
use std::{iter, time::Duration};

/// Builds two nodes that have each other as bootstrap nodes.
/// This is to be used only for testing, and a panic will happen if something goes wrong.
fn build_nodes() -> (Swarm<GenericProto>, Swarm<GenericProto>) {
    let mut out = Vec::with_capacity(2);

    let keypairs: Vec<_> = (0..2)
        .map(|_| identity::Keypair::generate_ed25519())
        .collect();
    let addrs: Vec<Multiaddr> = (0..2)
        .map(|_| {
            format!("/memory/{}", rand::random::<u64>())
                .parse()
                .unwrap()
        })
        .collect();

    for index in 0..2 {
        let keypair = keypairs[index].clone();

        let transport = MemoryTransport::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&keypair).unwrap())
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let (peerset, _) = sc_peerset::Peerset::from_config(sc_peerset::PeersetConfig {
            sets: vec![sc_peerset::SetConfig {
                in_peers: 25,
                out_peers: 25,
                bootnodes: if index == 0 {
                    keypairs
                        .iter()
                        .skip(1)
                        .map(|keypair| keypair.public().to_peer_id())
                        .collect()
                } else {
                    vec![]
                },
                reserved_nodes: Default::default(),
                reserved_only: false,
            }],
        });

        let behaviour = GenericProto::new(
            peerset,
            iter::once(("/foo".into(), Vec::new(), 1024 * 1024)),
        );
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            keypairs[index].public().to_peer_id(),
            SwarmConfig::with_executor(futures::executor::ThreadPool::new().unwrap()),
        );
        Swarm::listen_on(&mut swarm, addrs[index].clone()).unwrap();
        out.push(swarm);
    }

    // Add hardcoded addresses for all peers.
    for index in 0..out.len() {
        for other in 0..out.len() {
            if other != index {
                out[index]
                    .add_peer_address(keypairs[other].public().to_peer_id(), addrs[other].clone());
            }
        }
    }

    // Final output
    let mut out_iter = out.into_iter();
    let first = out_iter.next().unwrap();
    let second = out_iter.next().unwrap();
    (first, second)
}

#[test]
fn reconnect_after_disconnect() {
    // We connect two nodes together, then force a disconnect (through the API of the `Service`),
    // check that the disconnect worked, and finally check whether they successfully reconnect.

    let (mut service1, mut service2) = build_nodes();

    // For this test, the services can be in the following states.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum ServiceState {
        NotConnected,
        FirstConnec,
        Disconnected,
        ConnectedAgain,
    }
    let mut service1_state = ServiceState::NotConnected;
    let mut service2_state = ServiceState::NotConnected;

    futures::executor::block_on(async move {
        loop {
            // Grab next event from services.
            let event = {
                let s1 = service1.select_next_some();
                let s2 = service2.select_next_some();
                futures::pin_mut!(s1, s2);
                match future::select(s1, s2).await {
                    future::Either::Left((ev, _)) => future::Either::Left(ev),
                    future::Either::Right((ev, _)) => future::Either::Right(ev),
                }
            };

            match event {
                future::Either::Left(SwarmEvent::Behaviour(
                    GenericProtoOut::CustomProtocolOpen { .. },
                )) => match service1_state {
                    ServiceState::NotConnected => {
                        service1_state = ServiceState::FirstConnec;
                        if service2_state == ServiceState::FirstConnec {
                            service1.behaviour_mut().disconnect_peer(
                                Swarm::local_peer_id(&service2),
                                sc_peerset::SetId::from(0),
                            );
                        }
                    }
                    ServiceState::Disconnected => service1_state = ServiceState::ConnectedAgain,
                    ServiceState::FirstConnec | ServiceState::ConnectedAgain => panic!(),
                },
                future::Either::Left(SwarmEvent::Behaviour(
                    GenericProtoOut::CustomProtocolClosed { .. },
                )) => match service1_state {
                    ServiceState::FirstConnec => service1_state = ServiceState::Disconnected,
                    ServiceState::ConnectedAgain
                    | ServiceState::NotConnected
                    | ServiceState::Disconnected => panic!(),
                },
                future::Either::Right(SwarmEvent::Behaviour(
                    GenericProtoOut::CustomProtocolOpen { .. },
                )) => match service2_state {
                    ServiceState::NotConnected => {
                        service2_state = ServiceState::FirstConnec;
                        if service1_state == ServiceState::FirstConnec {
                            service1.behaviour_mut().disconnect_peer(
                                Swarm::local_peer_id(&service2),
                                sc_peerset::SetId::from(0),
                            );
                        }
                    }
                    ServiceState::Disconnected => service2_state = ServiceState::ConnectedAgain,
                    ServiceState::FirstConnec | ServiceState::ConnectedAgain => panic!(),
                },
                future::Either::Right(SwarmEvent::Behaviour(
                    GenericProtoOut::CustomProtocolClosed { .. },
                )) => match service2_state {
                    ServiceState::FirstConnec => service2_state = ServiceState::Disconnected,
                    ServiceState::ConnectedAgain
                    | ServiceState::NotConnected
                    | ServiceState::Disconnected => panic!(),
                },
                _ => {}
            }

            if service1_state == ServiceState::ConnectedAgain
                && matches!(
                    service2_state,
                    ServiceState::FirstConnec | ServiceState::ConnectedAgain
                )
            {
                break;
            }
        }

        // Now that the two services have disconnected and reconnected, wait for 3 seconds and
        // check whether they're still connected.
        let mut delay = futures_timer::Delay::new(Duration::from_secs(3));

        loop {
            // Grab next event from services.
            let event = {
                let s1 = service1.select_next_some();
                let s2 = service2.select_next_some();
                futures::pin_mut!(s1, s2);
                match future::select(future::select(s1, s2), &mut delay).await {
                    future::Either::Right(_) => break, // success
                    future::Either::Left((future::Either::Left((ev, _)), _)) => ev,
                    future::Either::Left((future::Either::Right((ev, _)), _)) => ev,
                }
            };

            match event {
                SwarmEvent::Behaviour(GenericProtoOut::CustomProtocolOpen { .. })
                | SwarmEvent::Behaviour(GenericProtoOut::CustomProtocolClosed { .. }) => panic!(),
                _ => {}
            }
        }
    });
}

#[test]
fn fallback_to_cached_external_address_for_pending_outbound() {
    let (peerset, _) = sc_peerset::Peerset::from_config(sc_peerset::PeersetConfig {
        sets: vec![sc_peerset::SetConfig {
            in_peers: 25,
            out_peers: 25,
            bootnodes: vec![],
            reserved_nodes: Default::default(),
            reserved_only: false,
        }],
    });

    let mut behaviour = GenericProto::new(
        peerset,
        iter::once(("/foo".into(), Vec::new(), 1024 * 1024)),
    );
    let peer_id = identity::Keypair::generate_ed25519().public().to_peer_id();
    let cached_addr: Multiaddr = "/memory/4242".parse().expect("valid memory addr");

    NetworkBehaviour::on_swarm_event(
        &mut behaviour,
        FromSwarm::NewExternalAddrOfPeer(NewExternalAddrOfPeer {
            peer_id,
            addr: &cached_addr,
        }),
    );

    let fallback = NetworkBehaviour::handle_pending_outbound_connection(
        &mut behaviour,
        ConnectionId::new_unchecked(1),
        Some(peer_id),
        &[],
        Endpoint::Dialer,
    )
    .expect("pending outbound should not fail");
    assert_eq!(fallback, vec![cached_addr.clone()]);

    let existing_addr: Multiaddr = "/memory/5252".parse().expect("valid memory addr");
    let passthrough = NetworkBehaviour::handle_pending_outbound_connection(
        &mut behaviour,
        ConnectionId::new_unchecked(2),
        Some(peer_id),
        std::slice::from_ref(&existing_addr),
        Endpoint::Dialer,
    )
    .expect("pending outbound should not fail");
    assert_eq!(passthrough, vec![existing_addr]);
}
