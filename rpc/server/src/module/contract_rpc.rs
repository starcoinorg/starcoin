// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::helpers::TransactionRequestFiller;
use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_account_api::AccountAsyncService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_resource_viewer::abi_resolver::ABIResolver;
use starcoin_rpc_api::contract_api::ContractApi;
use starcoin_rpc_api::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunTransactionRequest,
    FunctionIdView, ModuleIdView, StrView, StructTagView, TransactionOutputView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::ChainStateAsyncService;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::transaction::{DryRunTransaction, RawUserTransaction};
use starcoin_vm_types::abi::{ModuleABI, ScriptFunctionABI, StructABI};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
use std::str::FromStr;
use std::sync::Arc;

pub struct ContractRpcImpl<Account, Pool, State> {
    pub(crate) account: Option<Account>,
    pub(crate) pool: Pool,
    pub(crate) chain_state: State,
    pub(crate) node_config: Arc<NodeConfig>,
    playground: PlaygroudService,
    storage: Arc<Storage>,
}

impl<Account, Pool, State> ContractRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
{
    pub fn new(
        node_config: Arc<NodeConfig>,
        account: Option<Account>,
        pool: Pool,
        chain_state: State,
        playground: PlaygroudService,
        storage: Arc<Storage>,
    ) -> Self {
        Self {
            account,
            pool,
            chain_state,
            node_config,
            playground,
            storage,
        }
    }
    fn txn_request_filler(&self) -> TransactionRequestFiller<Account, Pool, State> {
        TransactionRequestFiller {
            account: self.account.clone(),
            pool: self.pool.clone(),
            chain_state: self.chain_state.clone(),
            node_config: self.node_config.clone(),
        }
    }
}

impl<Account, Pool, State> ContractApi for ContractRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
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
            function_id,
            type_args,
            args,
        } = call;
        let f = async move {
            let state_root = service.state_root().await?;
            let output = playground.call_contract(
                state_root,
                function_id.0.module,
                function_id.0.function,
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
        let f = async move {
            let state_root = service.state_root().await?;
            let DryRunTransactionRequest {
                transaction,
                sender_public_key,
            } = txn;

            let txn = txn_builder.fill_transaction(transaction).await?;

            let output = playground.dry_run(
                state_root,
                DryRunTransaction {
                    raw_txn: txn,
                    public_key: sender_public_key.0,
                },
            )?;
            Ok(output.1.into())
        }
        .map_err(map_err);
        Box::pin(f.boxed())
    }

    fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> FutureResult<TransactionOutputView> {
        let service = self.chain_state.clone();
        let playground = self.playground.clone();
        let f = async move {
            let state_root = service.state_root().await?;
            let raw_txn = RawUserTransaction::from_str(raw_txn.as_str())?;
            let output = playground.dry_run(
                state_root,
                DryRunTransaction {
                    raw_txn,
                    public_key: sender_public_key.0,
                },
            )?;
            Ok(output.1.into())
        }
        .map_err(map_err);
        Box::pin(f.boxed())
    }

    fn resolve_function(&self, function_id: FunctionIdView) -> FutureResult<ScriptFunctionABI> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        let fut = async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state)
                .resolve_function(&function_id.0.module, function_id.0.function.as_ident_str())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn resolve_struct_tag(&self, struct_tag: StructTagView) -> FutureResult<StructABI> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        let fut = async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state).resolve_struct_tag(&struct_tag.0)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn resolve_module(&self, module_id: ModuleIdView) -> FutureResult<ModuleABI> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        let fut = async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state).resolve_module(&module_id.0)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }
}
