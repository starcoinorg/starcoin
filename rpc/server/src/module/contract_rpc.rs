// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::helpers::TransactionRequestFiller;
use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_account_api::AccountAsyncService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_rpc_api::contract_api::ContractApi;
use starcoin_rpc_api::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunTransactionRequest,
    StrView, TransactionOutputView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::ChainStateAsyncService;
use starcoin_traits::ChainAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::transaction::DryRunTransaction;
use starcoin_vm_types::access_path::AccessPath;
use std::sync::Arc;

pub struct ContractRpcImpl<Account, Pool, State, Chain> {
    pub(crate) account: Option<Account>,
    pub(crate) pool: Pool,
    pub(crate) chain_state: State,
    pub(crate) chain: Chain,
    pub(crate) node_config: Arc<NodeConfig>,
    playground: PlaygroudService,
}

impl<Account, Pool, State, Chain> ContractRpcImpl<Account, Pool, State, Chain>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    pub fn new(
        node_config: Arc<NodeConfig>,
        account: Option<Account>,
        pool: Pool,
        chain_state: State,
        chain: Chain,
        playground: PlaygroudService,
    ) -> Self {
        Self {
            account,
            pool,
            chain_state,
            chain,
            node_config,
            playground,
        }
    }
    fn txn_request_filler(&self) -> TransactionRequestFiller<Account, Pool, State, Chain> {
        TransactionRequestFiller {
            account: self.account.clone(),
            pool: self.pool.clone(),
            chain_state: self.chain_state.clone(),
            chain: self.chain.clone(),
            node_config: self.node_config.clone(),
        }
    }
}

impl<Account, Pool, State, Chain> ContractApi for ContractRpcImpl<Account, Pool, State, Chain>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    fn get_code(&self, module_id: StrView<ModuleId>) -> FutureResult<Option<StrView<Vec<u8>>>> {
        let service = self.chain_state.clone();
        let f = async move {
            let code = service.get(AccessPath::from(&module_id.0)).await?;
            Ok(code.map(StrView))
        };
        Box::pin(f.map_err(map_err).boxed())
    }

    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> FutureResult<Option<AnnotatedMoveStructView>> {
        let service = self.chain_state.clone();
        let playground = self.playground.clone();
        let f = async move {
            let state_root = service.clone().state_root().await?;
            let data = service
                .get(AccessPath::resource_access_path(
                    addr,
                    resource_type.0.clone(),
                ))
                .await?;
            match data {
                None => Ok(None),
                Some(d) => {
                    let value =
                        playground.view_resource(state_root, &resource_type.0, d.as_slice())?;
                    Ok(Some(value.into()))
                }
            }
        };
        Box::pin(f.map_err(map_err).boxed())
    }
    fn call(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValueView>> {
        let service = self.chain_state.clone();
        let playground = self.playground.clone();
        let ContractCall {
            module_address,
            module_name,
            func,
            type_args,
            args,
        } = call;
        let f = async move {
            let state_root = service.state_root().await?;
            let output = playground.call_contract(
                state_root,
                module_address,
                module_name,
                func,
                type_args.into_iter().map(|v| v.0).collect(),
                args.into_iter().map(|v| v.0).collect(),
            )?;
            Ok(output.into_iter().map(Into::into).collect())
        }
        .map_err(map_err);
        Box::pin(f.boxed())
    }
    fn dry_run(&self, txn: DryRunTransactionRequest) -> FutureResult<TransactionOutputView> {
        let service = self.chain_state.clone();
        let txn_builder = self.txn_request_filler();
        let playground = self.playground.clone();
        let account_service = self.account.clone();
        let f = async move {
            let state_root = service.state_root().await?;
            let DryRunTransactionRequest {
                transaction,
                sender_public_key,
            } = txn;

            let txn = txn_builder.fill_transaction(transaction).await?;
            let sender_public_key = match sender_public_key {
                None => match account_service {
                    Some(account) => account
                        .get_account(txn.sender())
                        .await?
                        .map(|a| a.public_key)
                        .ok_or_else(|| {
                            anyhow::anyhow!("cannot fill public key of txn sender {}", txn.sender())
                        })?,
                    None => anyhow::bail!("account api is disabled"),
                },
                Some(p) => p.0,
            };

            let output = playground.dry_run(
                state_root,
                DryRunTransaction {
                    raw_txn: txn,
                    public_key: sender_public_key,
                },
            )?;
            Ok(output.1.into())
        }
        .map_err(map_err);
        Box::pin(f.boxed())
    }
}
