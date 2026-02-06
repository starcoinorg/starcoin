// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as StateClient;
use crate::FutureResult;
// copy from https://github.com/starcoinorg/starcoin/blob/bf5ec6e44a242e9dff5ac177c1565c64c6e4b0d0/rpc/api/src/state/mod.rs#L14 etc
use bytes::Bytes;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use openrpc_derive::openrpc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_types::{
    view::{
        AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
        StateWithProofView, StateWithTableItemProofView, StrView, StructTagView,
    },
    {account_address::AccountAddress, account_state::AccountState},
};
use starcoin_vm2_vm_types::{
    language_storage::{ModuleId, StructTag},
    state_store::{state_key::StateKey, table::TableHandle},
};
use std::sync::Arc;
#[openrpc]
pub trait StateApi {
    #[rpc(name = "state2.get")]
    fn get(&self, state_key: StateKey) -> FutureResult<Option<Bytes>>;

    /// Return state from StateTree storage directly by tree node key.
    #[rpc(name = "state2.get_state_node_by_node_hash")]
    fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> FutureResult<Option<Bytes>>;

    /// Return the Resource Or Code at the `access_path`, and provide a State Proof.
    #[rpc(name = "state2.get_with_proof")]
    fn get_with_proof(&self, state_key: StateKey) -> FutureResult<StateWithProofView>;

    /// Same as `state2.get_with_proof` but return `StateWithProof` in BCS serialize bytes.
    #[rpc(name = "state2.get_with_proof_raw")]
    fn get_with_proof_raw(&self, state_key: StateKey) -> FutureResult<StrView<Vec<u8>>>;

    #[rpc(name = "state2.get_account_state")]
    fn get_account_state(&self, address: AccountAddress) -> FutureResult<AccountState>;

    #[rpc(name = "state2.get_account_state_set")]
    fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> FutureResult<Option<AccountStateSetView>>;

    #[rpc(name = "state2.get_state_root")]
    fn get_state_root(&self) -> FutureResult<HashValue>;

    /// Return the Resource Or Code at the `access_path` and provide a State Proof at `state_root`
    #[rpc(name = "state2.get_with_proof_by_root")]
    fn get_with_proof_by_root(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> FutureResult<StateWithProofView>;

    /// Same as `state2.get_with_proof_by_root` but return `StateWithProof` in BCS serialize bytes.
    #[rpc(name = "state2.get_with_proof_by_root_raw")]
    fn get_with_proof_by_root_raw(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> FutureResult<StrView<Vec<u8>>>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    #[rpc(name = "state2.get_with_table_item_proof")]
    fn get_with_table_item_proof(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> FutureResult<StateWithTableItemProofView>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    #[rpc(name = "state2.get_with_table_item_proof_by_root")]
    fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> FutureResult<StateWithTableItemProofView>;

    /// get code of module
    #[rpc(name = "state2.get_code")]
    fn get_code(
        &self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> FutureResult<Option<CodeView>>;

    /// get resource data of `addr`
    #[rpc(name = "state2.get_resource")]
    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> FutureResult<Option<ResourceView>>;

    /// list resources data of `addr`
    #[rpc(name = "state2.list_resource")]
    fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> FutureResult<ListResourceView>;

    /// list resources data of `addr`
    #[rpc(name = "state2.list_code")]
    fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> FutureResult<ListCodeView>;
}

/// Build jsonrpsee methods from legacy `StateApi`.
pub fn state_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: StateApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("state2.get", |params, api, _| async move {
        let state_key: StateKey = params.one()?;
        api.get(state_key).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_state_node_by_node_hash", |params, api, _| async move {
        let key_hash: HashValue = params.one()?;
        api.get_state_node_by_node_hash(key_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_with_proof", |params, api, _| async move {
        let state_key: StateKey = params.one()?;
        api.get_with_proof(state_key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_with_proof_raw", |params, api, _| async move {
        let state_key: StateKey = params.one()?;
        api.get_with_proof_raw(state_key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_account_state", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.get_account_state(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_account_state_set", |params, api, _| async move {
        let (address, state_root): (AccountAddress, Option<HashValue>) = params.parse()?;
        api.get_account_state_set(address, state_root)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_state_root", |_, api, _| async move {
        api.get_state_root().await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_with_proof_by_root", |params, api, _| async move {
        let (state_key, state_root): (StateKey, HashValue) = params.parse()?;
        api.get_with_proof_by_root(state_key, state_root)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_with_proof_by_root_raw", |params, api, _| async move {
        let (state_key, state_root): (StateKey, HashValue) = params.parse()?;
        api.get_with_proof_by_root_raw(state_key, state_root)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_with_table_item_proof", |params, api, _| async move {
        let (handle, key): (TableHandle, Vec<u8>) = params.parse()?;
        api.get_with_table_item_proof(handle, key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method(
        "state2.get_with_table_item_proof_by_root",
        |params, api, _| async move {
            let (handle, key, state_root): (TableHandle, Vec<u8>, HashValue) = params.parse()?;
            api.get_with_table_item_proof_by_root(handle, key, state_root)
                .await
                .map_err(crate::map_jsonrpc_err)
        },
    )?;
    module.register_async_method("state2.get_code", |params, api, _| async move {
        let (module_id, option): (StrView<ModuleId>, Option<GetCodeOption>) = params.parse()?;
        api.get_code(module_id, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.get_resource", |params, api, _| async move {
        let (addr, resource_type, option): (AccountAddress, StrView<StructTag>, Option<GetResourceOption>) =
            params.parse()?;
        api.get_resource(addr, resource_type, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.list_resource", |params, api, _| async move {
        let (addr, option): (AccountAddress, Option<ListResourceOption>) = params.parse()?;
        api.list_resource(addr, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("state2.list_code", |params, api, _| async move {
        let (addr, option): (AccountAddress, Option<ListCodeOption>) = params.parse()?;
        api.list_code(addr, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct PrimaryFungibleStoreOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_code: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct GetResourceOption {
    pub decode: bool,
    pub state_root: Option<HashValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_fungible_store: Option<PrimaryFungibleStoreOption>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_fungible_store: Option<PrimaryFungibleStoreOption>,
}

impl Default for ListResourceOption {
    fn default() -> Self {
        Self {
            decode: false,
            state_root: None,
            start_index: 0,
            max_size: usize::MAX,
            resource_types: None,
            primary_fungible_store: None,
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
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
