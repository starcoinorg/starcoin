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
use starcoin_vm2_account_service::AccountService as AccountService2;
use starcoin_vm2_dev::playground::PlaygroudService as PlaygroudService2;
use starcoin_vm2_rpc_server::{
    account_rpc::AccountRpcImpl as AccountRpcImpl2,
    contract_rpc::ContractRpcImpl as ContractRpcImpl2, state_rpc::StateRpcImpl as StateRpcImpl2,
};
use starcoin_vm2_state_service::ChainStateService as ChainStateService2;
use starcoin_vm2_storage::Storage as Storage2;
use std::sync::Arc;

pub struct RpcServiceFactory;

// implement rpc service factory at node for avoid cycle dependency.
impl ServiceFactory<RpcService> for RpcServiceFactory {
    fn create(ctx: &mut ServiceContext<RpcService>) -> Result<RpcService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let genesis = ctx.get_shared::<Genesis>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
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
                    storage2.clone(),
                    service_ref.clone(),
                )
            });
        let txpool_service = ctx.get_shared::<TxPoolService>()?;
        let txpool_api = Some(TxPoolRpcImpl::new(txpool_service.clone()));

        let state_api = ctx
            .service_ref_opt::<ChainStateService>()?
            .map(|service_ref| StateRpcImpl::new(service_ref.clone(), storage.clone()));
        let state_api2 = ctx
            .service_ref_opt::<ChainStateService2>()?
            .map(|service_ref| StateRpcImpl2::new(service_ref.clone(), storage2.clone()));
        let chain_state_service = ctx.service_ref::<ChainStateService>()?.clone();
        let chain_state_service2 = ctx.service_ref::<ChainStateService2>()?.clone();
        let account_service = ctx.service_ref_opt::<AccountService>()?.cloned();
        let account_service2 = ctx.service_ref_opt::<AccountService2>()?.cloned();
        let account_api = account_service.clone().map(|service_ref| {
            AccountRpcImpl::new(
                config.clone(),
                service_ref,
                txpool_service.clone(),
                chain_state_service.clone(),
            )
        });
        let account_api2 = account_service2.clone().map(|service_ref| {
            AccountRpcImpl2::new(
                config.clone(),
                service_ref,
                txpool_service.clone(),
                chain_state_service2.clone(),
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
                txpool_service.clone(),
                chain_state_service,
                dev_playground,
                storage,
            )
        };
        let contract_api2 = {
            let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
            let dev_playground = PlaygroudService2::new(storage2.clone(), vm_metrics);
            ContractRpcImpl2::new(
                config.clone(),
                account_service2,
                txpool_service,
                chain_state_service2,
                dev_playground,
                storage2,
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
            account_api2,
            state_api2,
            Some(contract_api2),
        ))
    }
}
