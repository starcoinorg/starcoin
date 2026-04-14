// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::helpers::TransactionRequestFiller;
use crate::module::map_err;
use anyhow::format_err;
use jsonrpsee::core::{async_trait, RpcResult};
use starcoin_abi_decoder::{decode_move_value, DecodedMoveValue};
use starcoin_abi_resolver::ABIResolver;
use starcoin_abi_types::{FunctionABI, ModuleABI, StructInstantiation, TypeInstantiation};
use starcoin_account_api::AccountAsyncService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::{call_contract, PlaygroundService};
use starcoin_executor::VMMetrics;
use starcoin_resource_viewer::module_cache::ModuleCache;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::contract_api::ContractApiServer;
use starcoin_rpc_api::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunOutputView,
    DryRunTransactionRequest, FunctionIdView, ModuleIdView, StrView, StructTagView,
    TransactionOutputView, WriteOpValueView,
};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::transaction::{DryRunTransaction, RawUserTransaction, TransactionPayload};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
use std::str::FromStr;
use std::sync::Arc;

pub struct ContractRpcImpl<Account, Pool, State> {
    pub(crate) account: Option<Account>,
    pub(crate) pool: Pool,
    pub(crate) chain_state: State,
    pub(crate) node_config: Arc<NodeConfig>,
    playground: PlaygroundService,
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
        playground: PlaygroundService,
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

#[async_trait]
impl<Account, Pool, State> ContractApiServer for ContractRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
{
    async fn get_code(&self, module_id: StrView<ModuleId>) -> RpcResult<Option<StrView<Vec<u8>>>> {
        let code = self
            .chain_state
            .clone()
            .get(AccessPath::from(&module_id.0))
            .await
            .map_err(map_err)
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(code.map(StrView))
    }

    async fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> RpcResult<Option<AnnotatedMoveStructView>> {
        let service = self.chain_state.clone();
        let playground = self.playground.clone();
        async move {
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
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }
    async fn call(&self, call: ContractCall) -> RpcResult<Vec<AnnotatedMoveValueView>> {
        let service = self.chain_state.clone();
        let playground = self.playground.clone();
        let ContractCall {
            function_id,
            type_args,
            args,
        } = call;
        async move {
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
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn call_v2(&self, call: ContractCall) -> RpcResult<Vec<DecodedMoveValue>> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        let ContractCall {
            function_id,
            type_args,
            args,
        } = call;
        let metrics = self.playground.metrics.clone();
        async move {
            let state_root = service.state_root().await?;
            let state = ChainStateDB::new(storage, Some(state_root));
            let output = call_contract(
                &state,
                function_id.0.module,
                function_id.0.function.as_str(),
                type_args.into_iter().map(|v| v.0).collect(),
                args.into_iter().map(|v| v.0).collect(),
                metrics,
            )?;
            let annotator = MoveValueAnnotator::new(&state);
            output
                .into_iter()
                .map(|(ty, v)| annotator.view_value(&ty, &v).map(Into::into))
                .collect::<anyhow::Result<Vec<_>>>()
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn dry_run(&self, txn: DryRunTransactionRequest) -> RpcResult<DryRunOutputView> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        let txn_builder = self.txn_request_filler();
        let metrics = self.playground.metrics.clone();
        async move {
            let state_root = service.state_root().await?;
            let DryRunTransactionRequest {
                transaction,
                sender_public_key,
            } = txn;

            let txn = txn_builder.fill_transaction(transaction).await?;
            let state_view = ChainStateDB::new(storage, Some(state_root));
            dry_run(
                &state_view,
                DryRunTransaction {
                    raw_txn: txn,
                    public_key: sender_public_key.0,
                },
                metrics,
            )
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> RpcResult<DryRunOutputView> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        let metrics = self.playground.metrics.clone();
        async move {
            let state_root = service.state_root().await?;
            let raw_txn = RawUserTransaction::from_str(raw_txn.as_str())?;
            let state_view = ChainStateDB::new(storage, Some(state_root));
            dry_run(
                &state_view,
                DryRunTransaction {
                    raw_txn,
                    public_key: sender_public_key.0,
                },
                metrics,
            )
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn resolve_function(&self, function_id: FunctionIdView) -> RpcResult<FunctionABI> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state)
                .resolve_function(&function_id.0.module, function_id.0.function.as_ident_str())
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn resolve_module_function_index(
        &self,
        module_id: ModuleIdView,
        function_idx: u16,
    ) -> RpcResult<FunctionABI> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state).resolve_module_function_index(&module_id.0, function_idx)
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn resolve_struct(&self, struct_tag: StructTagView) -> RpcResult<StructInstantiation> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state).resolve_struct_tag(&struct_tag.0)
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }

    async fn resolve_module(&self, module_id: ModuleIdView) -> RpcResult<ModuleABI> {
        let service = self.chain_state.clone();
        let storage = self.storage.clone();
        async move {
            let state = ChainStateDB::new(storage, Some(service.state_root().await?));
            ABIResolver::new(&state).resolve_module(&module_id.0)
        }
        .await
        .map_err(map_err)
        .map_err(crate::module::map_jsonrpc_err)
    }
}

pub fn dry_run<S: StateView>(
    state_view: &S,
    txn: DryRunTransaction,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<DryRunOutputView> {
    let (vm_status, output) = starcoin_dev::playground::dry_run(state_view, txn.clone(), metrics)?;
    let vm_status_explain = vm_status_translator::explain_vm_status(state_view, vm_status)?;
    let mut txn_output: TransactionOutputView = output.into();

    let resolver = {
        let module_cache = ModuleCache::new();
        // If the txn is package txn, we need to use modules in the package to resolve transaction output.
        if let TransactionPayload::Package(p) = txn.raw_txn.into_payload() {
            let modules = p
                .modules()
                .iter()
                .map(|m| CompiledModule::deserialize(m.code()))
                .collect::<Result<Vec<_>, _>>()?;
            for m in modules {
                module_cache.insert(m.self_id(), m);
            }
        }
        ABIResolver::new_with_module_cache(state_view, module_cache)
    };
    for action in txn_output.write_set.iter_mut() {
        let access_path = action.access_path.clone();
        if let Some(value) = &mut action.value {
            match value {
                WriteOpValueView::Code(view) => {
                    view.abi = Some(resolver.resolve_module_code(view.code.0.as_slice())?);
                }
                WriteOpValueView::Resource(view) => {
                    let struct_tag = access_path.path.as_struct_tag().ok_or_else(|| {
                        format_err!("invalid resource access path: {}", access_path)
                    })?;
                    let struct_abi = resolver.resolve_struct_tag(struct_tag)?;
                    view.json = Some(decode_move_value(
                        &TypeInstantiation::Struct(Box::new(struct_abi)),
                        view.raw.0.as_slice(),
                    )?)
                }
            }
        }
    }
    Ok(DryRunOutputView {
        explained_status: vm_status_explain,
        txn_output,
    })
}
