// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use network_p2p::NetworkWorker;
use network_types::peer_info::PeerInfo;
use starcoin_config::{ChainNetwork, NetworkConfig};
use starcoin_network::network_p2p_handle::Networkp2pHandle;
use starcoin_network::{build_network_worker, NotificationMessage};
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_types::startup_info::ChainInfo;
use std::sync::Arc;

pub fn build_lighting_network(
    net: &ChainNetwork,
    network_config: &NetworkConfig,
) -> Result<(PeerInfo, NetworkWorker<Networkp2pHandle>)> {
    let genesis = starcoin_genesis::Genesis::load_or_build(net)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
    let chain_info = genesis.execute_genesis_block(net, storage.clone())?;
    let chain_state_info = ChainInfo::new(
        chain_info.chain_id(),
        chain_info.genesis_hash(),
        chain_info.status().clone(),
    );
    build_network_worker(
        network_config,
        chain_state_info,
        NotificationMessage::protocols(),
        None,
        None,
    )
}
