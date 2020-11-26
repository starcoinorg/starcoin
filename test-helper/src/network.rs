// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::{BlockStore, Storage};
use std::sync::{Arc, Mutex};

use network_api::{MultiaddrWithPeerId, PeerMessageHandler};
use starcoin_block_relayer_api::PeerCmpctBlockEvent;
pub use starcoin_network::NetworkAsyncService;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_tx_relay::PeerTransactions;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};

#[derive(Clone, Default)]
pub struct MockPeerMessageHandler {
    pub txns: Arc<Mutex<Vec<PeerTransactions>>>,
    pub blocks: Arc<Mutex<Vec<PeerCmpctBlockEvent>>>,
}

impl PeerMessageHandler for MockPeerMessageHandler {
    fn handle_transaction(&self, transaction: PeerTransactions) {
        self.txns.lock().unwrap().push(transaction);
    }

    fn handle_block(&self, block: PeerCmpctBlockEvent) {
        self.blocks.lock().unwrap().push(block);
    }
}

pub async fn build_network<H>(
    seed: Option<MultiaddrWithPeerId>,
    rpc_service_mocker: Option<impl MockHandler<NetworkRpcService> + 'static>,
    peer_message_handler: H,
) -> Result<(
    NetworkAsyncService,
    Arc<NodeConfig>,
    Arc<Storage>,
    ServiceRef<RegistryService>,
)>
where
    H: PeerMessageHandler + 'static,
{
    let registry = RegistryService::launch();

    let mut config = NodeConfig::random_for_test();
    if let Some(seed) = seed {
        config.network.seeds = vec![seed];
    }
    let node_config = Arc::new(config);
    let (storage, startup_info, genesis_hash) = Genesis::init_storage_for_test(node_config.net())?;

    let head_block_hash = startup_info.main;
    let head_block_header = storage
        .get_block_header_by_hash(head_block_hash)?
        .ok_or_else(|| format_err!("can't get block by hash {}", head_block_hash))?;
    let head_block_info = storage
        .get_block_info(head_block_hash)?
        .ok_or_else(|| format_err!("can't get block info by hash {}", head_block_hash))?;

    let chain_info = ChainInfo::new(
        node_config.net().chain_id(),
        genesis_hash,
        ChainStatus::new(head_block_header, head_block_info.total_difficulty),
    );

    registry.put_shared(node_config.clone()).await?;
    registry.put_shared(storage.clone()).await?;

    let bus = registry.service_ref::<BusService>().await?;
    let network_rpc_service = if let Some(mocker) = rpc_service_mocker {
        registry.register_mocker(mocker).await?
    } else {
        registry.register::<NetworkRpcService>().await?
    };

    Ok((
        NetworkAsyncService::start(
            node_config.clone(),
            chain_info,
            bus,
            network_rpc_service,
            peer_message_handler,
        )?,
        node_config,
        storage,
        registry,
    ))
}
