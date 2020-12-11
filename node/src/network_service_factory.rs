// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_message_handler::NodePeerMessageHandler;
use anyhow::{format_err, Result};
use starcoin_block_relayer::BlockRelayer;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_network::{NetworkActorService, NetworkServiceRef};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::{ServiceContext, ServiceFactory};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_txpool::TxPoolActorService;
use starcoin_types::peer_info::RpcInfo;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::sync::Arc;

pub struct NetworkServiceFactory;

impl ServiceFactory<NetworkActorService> for NetworkServiceFactory {
    fn create(ctx: &mut ServiceContext<NetworkActorService>) -> Result<NetworkActorService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let genesis = ctx.get_shared::<Genesis>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let rpc_info = RpcInfo::new(starcoin_network_rpc_api::gen_client::get_rpc_info());
        let txpool_service = ctx.service_ref::<TxPoolActorService>()?.clone();
        let block_relayer = ctx.service_ref::<BlockRelayer>()?.clone();
        let network_rpc_service = ctx.service_ref::<NetworkRpcService>()?.clone();
        let peer_message_handle = NodePeerMessageHandler::new(txpool_service, block_relayer);

        //TODO move get chain info to storage.
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Can not find startup_info"))?;
        let genesis_hash = genesis.block().header().id();
        let head_block_hash = startup_info.main;
        let head_block_header = storage
            .get_block_header_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block by hash {}", head_block_hash))?;

        let head_block_info = storage
            .get_block_info(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block info by hash {}", head_block_hash))?;

        let chain_info = ChainInfo::new(
            config.net().chain_id(),
            genesis_hash,
            ChainStatus::new(head_block_header, head_block_info.total_difficulty),
        );

        let actor_service = NetworkActorService::new(
            config,
            chain_info,
            Some((rpc_info, network_rpc_service)),
            peer_message_handle,
        )?;
        let network_service = actor_service.network_service();
        let network_async_service = NetworkServiceRef::new(network_service, ctx.self_ref());
        ctx.put_shared(network_async_service)?;
        Ok(actor_service)
    }
}
