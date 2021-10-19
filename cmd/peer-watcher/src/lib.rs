// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use network_p2p::NetworkWorker;
use starcoin_config::{ChainNetwork, NetworkConfig};
use starcoin_network::{build_network_worker, NotificationMessage};
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_types::peer_info::PeerInfo;
use std::sync::Arc;

pub fn build_lighting_network(
    net: &ChainNetwork,
    network_config: &NetworkConfig,
) -> Result<(PeerInfo, NetworkWorker)> {
    let genesis = starcoin_genesis::Genesis::load_or_build(net)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
    let chain_info = genesis.execute_genesis_block(net, storage)?;
    build_network_worker(
        network_config,
        chain_info,
        NotificationMessage::protocols(),
        None,
        None,
    )
}
