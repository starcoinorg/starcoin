// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node::NodeService;
use anyhow::Result;
use starcoin_account_service::AccountService;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_executor::VMMetrics;
use starcoin_genesis::Genesis;
use starcoin_logger::LoggerHandle;
use starcoin_miner::MinerService;
use starcoin_network::NetworkServiceRef;
use starcoin_rpc_server::module::{
    AccountRpcImpl, ChainRpcImpl, ContractRpcImpl, DebugRpcImpl, MinerRpcImpl,
    NetworkManagerRpcImpl, NodeManagerRpcImpl, NodeRpcImpl, PubSubImpl, PubSubService,
    StateRpcImpl, SyncManagerRpcImpl, TxPoolRpcImpl,
};
use starcoin_rpc_server::service::RpcService;
use starcoin_service_registry::{ServiceContext, ServiceFactory};
use starcoin_state_service::ChainStateService;
use starcoin_storage::Storage;
use starcoin_sync::sync::SyncService;
use starcoin_txpool::TxPoolService;
use std::sync::Arc;

pub struct RpcServiceFactory;

// implement rpc service factory at node for avoid cycle dependency.
impl ServiceFactory<RpcService> for RpcServiceFactory {
    fn create(ctx: &mut ServiceContext<RpcService>) -> Result<RpcService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let genesis = ctx.get_shared::<Genesis>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let log_handler = ctx.get_shared::<Arc<LoggerHandle>>()?;
        let network_service = ctx.get_shared::<NetworkServiceRef>()?;
        let node_api = NodeRpcImpl::new(config.clone(), Some(network_service.clone()));
        let node_manager_api = ctx
            .service_ref_opt::<NodeService>()?
            .map(|service_ref| NodeManagerRpcImpl::new(service_ref.clone()));
        let sync_manager_api = ctx
            .service_ref_opt::<SyncService>()?
            .map(|service_ref| SyncManagerRpcImpl::new(service_ref.clone()));
        let network_manager_api = NetworkManagerRpcImpl::new(network_service);
        let chain_api = ctx
            .service_ref_opt::<ChainReaderService>()?
            .map(|service_ref| {
                ChainRpcImpl::new(
                    config.clone(),
                    genesis.block().id(),
                    storage.clone(),
                    service_ref.clone(),
                )
            });
        let txpool_service = ctx.get_shared::<TxPoolService>()?;
        let txpool_api = Some(TxPoolRpcImpl::new(txpool_service.clone()));

        let state_api = ctx
            .service_ref_opt::<ChainStateService>()?
            .map(|service_ref| StateRpcImpl::new(service_ref.clone(), storage.clone()));
        let chain_state_service = ctx.service_ref::<ChainStateService>()?.clone();
        let account_service = ctx.service_ref_opt::<AccountService>()?.cloned();
        let account_api = account_service.clone().map(|service_ref| {
            AccountRpcImpl::new(
                config.clone(),
                service_ref,
                txpool_service.clone(),
                chain_state_service.clone(),
            )
        });
        let pubsub_service = ctx.service_ref::<PubSubService>()?.clone();
        let pubsub_api = Some(PubSubImpl::new(pubsub_service));
        let debug_api = Some(DebugRpcImpl::new(
            config.clone(),
            log_handler,
            ctx.bus_ref().clone(),
        ));
        let miner_api = ctx
            .service_ref_opt::<MinerService>()?
            .map(|service_ref| MinerRpcImpl::new(service_ref.clone()));

        let contract_api = {
            let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
            let dev_playground = PlaygroudService::new(storage.clone(), vm_metrics);
            ContractRpcImpl::new(
                config.clone(),
                account_service,
                txpool_service,
                chain_state_service,
                dev_playground,
                storage,
            )
        };

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
            Some(contract_api),
        ))
    }
}
