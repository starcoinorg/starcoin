// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as ContractClient;
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use openrpc_derive::openrpc;
use starcoin_vm2_abi_decoder::DecodedMoveValue;
use starcoin_vm2_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_vm2_types::view::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunOutputView,
    DryRunTransactionRequest, FunctionIdView, ModuleIdView, StrView, StructTagView,
};
use starcoin_vm2_vm_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    transaction::authenticator::AccountPublicKey,
};
use std::sync::Arc;
// copy from https://github.com/starcoinorg/starcoin/blob/bf5ec6e44a242e9dff5ac177c1565c64c6e4b0d0/rpc/api/src/contract_api.rs#L14 etc
#[openrpc]
pub trait ContractApi {
    /// get code of module
    #[rpc(name = "contract2.get_code")]
    fn get_code(&self, module_id: StrView<ModuleId>) -> FutureResult<Option<StrView<Vec<u8>>>>;

    /// get resource data of `addr`
    #[rpc(name = "contract2.get_resource")]
    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> FutureResult<Option<AnnotatedMoveStructView>>;

    /// Call a move contract, return returned move values.
    #[rpc(name = "contract2.call")]
    fn call(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValueView>>;

    /// Call a move contract, return move values.
    #[rpc(name = "contract2.call_v2")]
    fn call_v2(&self, call: ContractCall) -> FutureResult<Vec<DecodedMoveValue>>;

    #[rpc(name = "contract2.dry_run")]
    fn dry_run(&self, txn: DryRunTransactionRequest) -> FutureResult<DryRunOutputView>;

    /// Dry run RawUserTransaction, the raw_txn parameter is RawUserTransaction's hex
    #[rpc(name = "contract2.dry_run_raw")]
    fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> FutureResult<DryRunOutputView>;
    #[rpc(name = "contract2.resolve_function")]
    fn resolve_function(&self, function_id: FunctionIdView) -> FutureResult<FunctionABI>;
    #[rpc(name = "contract2.resolve_module_function_index")]
    fn resolve_module_function_index(
        &self,
        module_id: ModuleIdView,
        function_index: u16,
    ) -> FutureResult<FunctionABI>;
    #[rpc(name = "contract2.resolve_struct")]
    fn resolve_struct(&self, struct_tag: StructTagView) -> FutureResult<StructInstantiation>;
    #[rpc(name = "contract2.resolve_module")]
    fn resolve_module(&self, module_id: ModuleIdView) -> FutureResult<ModuleABI>;
}

/// Build jsonrpsee methods from legacy `ContractApi`.
pub fn contract_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: ContractApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("contract2.get_code", |params, api, _| async move {
        let module_id: StrView<ModuleId> = params.one()?;
        api.get_code(module_id).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.get_resource", |params, api, _| async move {
        let (addr, resource_type): (AccountAddress, StrView<StructTag>) = params.parse()?;
        api.get_resource(addr, resource_type)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.call", |params, api, _| async move {
        let call: ContractCall = params.one()?;
        api.call(call).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.call_v2", |params, api, _| async move {
        let call: ContractCall = params.one()?;
        api.call_v2(call).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.dry_run", |params, api, _| async move {
        let txn: DryRunTransactionRequest = params.one()?;
        api.dry_run(txn).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.dry_run_raw", |params, api, _| async move {
        let (raw_txn, sender_public_key): (String, StrView<AccountPublicKey>) = params.parse()?;
        api.dry_run_raw(raw_txn, sender_public_key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.resolve_function", |params, api, _| async move {
        let function_id: FunctionIdView = params.one()?;
        api.resolve_function(function_id)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method(
        "contract2.resolve_module_function_index",
        |params, api, _| async move {
            let (module_id, function_index): (ModuleIdView, u16) = params.parse()?;
            api.resolve_module_function_index(module_id, function_index)
                .await
                .map_err(crate::map_jsonrpc_err)
        },
    )?;
    module.register_async_method("contract2.resolve_struct", |params, api, _| async move {
        let struct_tag: StructTagView = params.one()?;
        api.resolve_struct(struct_tag)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("contract2.resolve_module", |params, api, _| async move {
        let module_id: ModuleIdView = params.one()?;
        api.resolve_module(module_id)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}
