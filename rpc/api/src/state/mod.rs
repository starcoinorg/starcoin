// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type StateClient = jsonrpsee::async_client::Client;
use crate::types::{
    AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView,
};
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use starcoin_vm_types::state_store::table::TableHandle;
use std::sync::Arc;
pub trait StateApi {
    fn get(&self, access_path: AccessPath) -> FutureResult<Option<Vec<u8>>>;

    /// Return state from StateTree storage directly by tree node key.
    fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> FutureResult<Option<Vec<u8>>>;

    /// Return the Resource Or Code at the `access_path`, and provide a State Proof.
    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProofView>;

    /// Same as `state.get_with_proof` but return `StateWithProof` in BCS serialize bytes.
    fn get_with_proof_raw(&self, access_path: AccessPath) -> FutureResult<StrView<Vec<u8>>>;
    fn get_account_state(&self, address: AccountAddress) -> FutureResult<Option<AccountState>>;
    fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> FutureResult<Option<AccountStateSetView>>;
    fn get_state_root(&self) -> FutureResult<HashValue>;

    /// Return the Resource Or Code at the `access_path` and provide a State Proof at `state_root`
    fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StateWithProofView>;

    /// Same as `state.get_with_proof_by_root` but return `StateWithProof` in BCS serialize bytes.
    fn get_with_proof_by_root_raw(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StrView<Vec<u8>>>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    fn get_with_table_item_proof(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> FutureResult<StateWithTableItemProofView>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> FutureResult<StateWithTableItemProofView>;

    /// get code of module
    fn get_code(
        &self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> FutureResult<Option<CodeView>>;

    /// get resource data of `addr`
    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> FutureResult<Option<ResourceView>>;

    /// list resources data of `addr`
    fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> FutureResult<ListResourceView>;

    /// list resources data of `addr`
    fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> FutureResult<ListCodeView>;
}

/// Build jsonrpsee methods from legacy `StateApi`.
///
/// This keeps the existing `StateApi` trait unchanged and enables incremental
/// server runtime migration to jsonrpsee.
pub fn state_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: StateApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("state.get", |params, api, _| async move {
        let access_path: AccessPath = params.one()?;
        api.get(access_path).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_state_node_by_node_hash", |params, api, _| async move {
        let key_hash: HashValue = params.one()?;
        api.get_state_node_by_node_hash(key_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_with_proof", |params, api, _| async move {
        let access_path: AccessPath = params.one()?;
        api.get_with_proof(access_path)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_with_proof_raw", |params, api, _| async move {
        let access_path: AccessPath = params.one()?;
        api.get_with_proof_raw(access_path)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_account_state", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.get_account_state(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_account_state_set", |params, api, _| async move {
        let (address, state_root): (AccountAddress, Option<HashValue>) = params.parse()?;
        api.get_account_state_set(address, state_root)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_state_root", |_, api, _| async move {
        api.get_state_root().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_with_proof_by_root", |params, api, _| async move {
        let (access_path, state_root): (AccessPath, HashValue) = params.parse()?;
        api.get_with_proof_by_root(access_path, state_root)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_with_proof_by_root_raw", |params, api, _| async move {
        let (access_path, state_root): (AccessPath, HashValue) = params.parse()?;
        api.get_with_proof_by_root_raw(access_path, state_root)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_with_table_item_proof", |params, api, _| async move {
        let (handle, key): (TableHandle, Vec<u8>) = params.parse()?;
        api.get_with_table_item_proof(handle, key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method(
        "state.get_with_table_item_proof_by_root",
        |params, api, _| async move {
            let (handle, key, state_root): (TableHandle, Vec<u8>, HashValue) = params.parse()?;
            api.get_with_table_item_proof_by_root(handle, key, state_root)
                .await
                .map_err(crate::map_jsonrpc_err)
        },
    )?;

    module.register_async_method("state.get_code", |params, api, _| async move {
        let (module_id, option): (StrView<ModuleId>, Option<GetCodeOption>) = params.parse()?;
        api.get_code(module_id, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.get_resource", |params, api, _| async move {
        let (addr, resource_type, option): (AccountAddress, StrView<StructTag>, Option<GetResourceOption>) =
            params.parse()?;
        api.get_resource(addr, resource_type, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.list_resource", |params, api, _| async move {
        let (addr, option): (AccountAddress, Option<ListResourceOption>) = params.parse()?;
        api.list_resource(addr, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("state.list_code", |params, api, _| async move {
        let (addr, option): (AccountAddress, Option<ListCodeOption>) = params.parse()?;
        api.list_code(addr, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct GetResourceOption {
    pub decode: bool,
    pub state_root: Option<HashValue>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct GetCodeOption {
    pub resolve: bool,
    pub state_root: Option<HashValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct ListResourceOption {
    pub decode: bool,
    /// The state tree root, default is the latest block state root
    pub state_root: Option<HashValue>,
    pub start_index: usize,
    pub max_size: usize,
    pub resource_types: Option<Vec<StructTagView>>,
}

impl Default for ListResourceOption {
    fn default() -> Self {
        Self {
            decode: false,
            state_root: None,
            start_index: 0,
            max_size: usize::MAX,
            resource_types: None,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct ListCodeOption {
    pub resolve: bool,
    /// The state tree root, default is the latest block state root
    pub state_root: Option<HashValue>,
    //TODO support filter by type and pagination
}
