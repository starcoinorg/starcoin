// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::BCSCodec;
use clap::Parser;
use futures::StreamExt;
use network_p2p::Event;
use network_types::peer_info::PeerInfo;
use starcoin_config::{NodeConfig, StarcoinOpt};
use starcoin_peer_watcher::build_lighting_network;
use starcoin_types::startup_info::ChainInfo;

/// A lighting node, connect to peer to peer network, and monitor peers.
fn main() {
    let _logger = starcoin_logger::init();
    let opt: StarcoinOpt = StarcoinOpt::parse();
    let config = NodeConfig::load_with_opt(&opt).unwrap();
    let (peer_info, worker) = build_lighting_network(config.net(), &config.network).unwrap();
    println!("Self peer_info: {:?}", peer_info);
    let service = worker.service().clone();
    async_std::task::spawn(worker);
    let stream = service.event_stream("peer_watcher");
    futures::executor::block_on(async move {
        stream
            .filter_map(|event| async move {
                match event {
                    Event::NotificationStreamOpened {
                        remote,
                        protocol: _,
                        generic_data,
                        notif_protocols,
                        rpc_protocols,
                        version_string,
                    } => match ChainInfo::decode(&generic_data) {
                        Ok(chain_info) => Some(PeerInfo::new(
                            remote.into(),
                            chain_info,
                            notif_protocols,
                            rpc_protocols,
                            version_string,
                        )),
                        Err(error) => {
                            println!("failed to decode generic message for the reason: {}", error);
                            None
                        }
                    },
                    _ => None,
                }
            })
            .for_each(|peer| async move {
                //TODO save peer info to database or post to a webhook
                // get peer's more info from network state, such as ip address, version etc.
                println!("Find peer: {:?}", peer)
            })
            .await;
    });
}
