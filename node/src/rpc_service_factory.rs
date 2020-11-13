// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node::NodeService;
use anyhow::Result;
use starcoin_account_service::AccountService;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_logger::LoggerHandle;
use starcoin_miner::MinerService;
use starcoin_network::NetworkAsyncService;
use starcoin_rpc_server::module::{
    AccountRpcImpl, ChainRpcImpl, DebugRpcImpl, DevRpcImpl, MinerRpcImpl, NetworkManagerRpcImpl,
    NodeManagerRpcImpl, NodeRpcImpl, PubSubImpl, PubSubService, StateRpcImpl, SyncManagerRpcImpl,
    TxPoolRpcImpl,
};
use starcoin_rpc_server::service::RpcService;
use starcoin_service_registry::{ServiceContext, ServiceFactory};
use starcoin_state_service::ChainStateService;
use starcoin_storage::Storage;
use starcoin_sync::sync2::SyncService2;
use starcoin_txpool::TxPoolService;
use std::sync::Arc;

pub struct RpcServiceFactory;

// implement rpc service factory at node for avoid cycle dependency.
impl ServiceFactory<RpcService> for RpcServiceFactory {
    fn create(ctx: &mut ServiceContext<RpcService>) -> Result<RpcService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let bus = ctx.bus_ref().clone();
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let log_handler = ctx.get_shared::<Arc<LoggerHandle>>()?;
        let network_service = ctx.get_shared::<NetworkAsyncService>()?;
        let node_api = NodeRpcImpl::new(config.clone(), Some(network_service.clone()));
        let node_manager_api = ctx
            .service_ref_opt::<NodeService>()?
            .map(|service_ref| NodeManagerRpcImpl::new(service_ref.clone()));
        let sync_manager_api = ctx
            .service_ref_opt::<SyncService2>()?
            .map(|service_ref| SyncManagerRpcImpl::new(service_ref.clone()));
        let network_manager_api = NetworkManagerRpcImpl::new(network_service);
        let chain_api = ctx
            .service_ref_opt::<ChainReaderService>()?
            .map(|service_ref| ChainRpcImpl::new(service_ref.clone()));
        let txpool_service = ctx.get_shared::<TxPoolService>()?;
        let txpool_api = Some(TxPoolRpcImpl::new(txpool_service.clone()));
        let account_api = ctx
            .service_ref_opt::<AccountService>()?
            .map(|service_ref| AccountRpcImpl::new(service_ref.clone()));
        let state_api = ctx
            .service_ref_opt::<ChainStateService>()?
            .map(|service_ref| StateRpcImpl::new(service_ref.clone()));
        let pubsub_service = PubSubService::new(bus, txpool_service);
        let pubsub_api = Some(PubSubImpl::new(pubsub_service));
        let debug_api = Some(DebugRpcImpl::new(config.clone(), log_handler));
        let miner_api = ctx
            .service_ref_opt::<MinerService>()?
            .map(|service_ref| MinerRpcImpl::new(service_ref.clone()));

        let dev_api = ctx
            .service_ref_opt::<ChainStateService>()?
            .map(|service_ref| {
                let dev_playground = PlaygroudService::new(storage);
                DevRpcImpl::new(service_ref.clone(), dev_playground)
            });
        Ok(RpcService::new_with_api(
            config,
            node_api,
            node_manager_api,
            sync_manager_api,
            Some(network_manager_api),
            chain_api,
            txpool_api,
            account_api,
            state_api,
            pubsub_api,
            debug_api,
            miner_api,
            dev_api,
        ))
    }
}
