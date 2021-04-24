// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_message_handler::NodePeerMessageHandler;
use anyhow::{format_err, Result};
use starcoin_block_relayer::BlockRelayer;
use starcoin_config::NodeConfig;
use starcoin_network::{NetworkActorService, NetworkServiceRef};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::{ServiceContext, ServiceFactory};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync::announcement::AnnouncementService;
use starcoin_txpool::TxPoolActorService;
use std::sync::Arc;

pub struct NetworkServiceFactory;

impl ServiceFactory<NetworkActorService> for NetworkServiceFactory {
    fn create(ctx: &mut ServiceContext<NetworkActorService>) -> Result<NetworkActorService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let rpc_info = starcoin_network_rpc_api::RPC_INFO.clone();
        let txpool_service = ctx.service_ref::<TxPoolActorService>()?.clone();
        let block_relayer = ctx.service_ref::<BlockRelayer>()?.clone();
        let network_rpc_service = ctx.service_ref::<NetworkRpcService>()?.clone();
        let announcement_service = ctx.service_ref::<AnnouncementService>()?.clone();
        let peer_message_handle =
            NodePeerMessageHandler::new(txpool_service, block_relayer, announcement_service);

        let chain_info = storage
            .get_chain_info()?
            .ok_or_else(|| format_err!("Can not get chain info."))?;
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
