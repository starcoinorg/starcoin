// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::StreamExt;
use network_p2p::Event;
use starcoin_config::{NodeConfig, StarcoinOpt};
use starcoin_peer_watcher::build_lighting_network;
use starcoin_types::peer_info::PeerInfo;
use structopt::StructOpt;

/// A lighting node, connect to peer to peer network, and monitor peers.
fn main() {
    let _logger = starcoin_logger::init();
    let opt: StarcoinOpt = StarcoinOpt::from_args();
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
                        info,
                        notif_protocols,
                        rpc_protocols,
                    } => Some(PeerInfo::new(
                        remote.into(),
                        *info,
                        notif_protocols,
                        rpc_protocols,
                    )),
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
