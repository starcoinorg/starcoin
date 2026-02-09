// This file is part of Substrate.
//
// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{Boxed, MemoryTransport},
        upgrade,
    },
    dns, identity, noise, tcp, PeerId, Transport,
};
use std::{sync::Arc, time::Duration};

#[derive(Debug, Default)]
pub struct BandwidthSinks;

impl BandwidthSinks {
    pub fn total_inbound(&self) -> u64 {
        0
    }

    pub fn total_outbound(&self) -> u64 {
        0
    }
}

/// Builds the base transport used by `network-p2p`.
pub fn build_transport(
    keypair: identity::Keypair,
    memory_only: bool,
) -> (Boxed<(PeerId, StreamMuxerBox)>, Arc<BandwidthSinks>) {
    let noise_config = noise::Config::new(&keypair)
        .expect("Noise static DH key generation from identity keypair must succeed");
    let yamux_config = libp2p::yamux::Config::default();

    let transport = if memory_only {
        MemoryTransport::default()
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise_config)
            .multiplex(yamux_config)
            .timeout(Duration::from_secs(20))
            .boxed()
    } else {
        let tcp_config = tcp::Config::new().nodelay(true);
        let tcp_transport = tcp::tokio::Transport::new(tcp_config);
        let base = match dns::tokio::Transport::system(tcp_transport) {
            Ok(dns_transport) => dns_transport.boxed(),
            Err(_) => tcp::tokio::Transport::new(tcp::Config::new().nodelay(true)).boxed(),
        };

        base.upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&keypair).expect("noise config must succeed"))
            .multiplex(libp2p::yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed()
    };

    (transport, Arc::new(BandwidthSinks))
}
