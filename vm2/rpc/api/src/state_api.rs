// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

// copy from https://github.com/starcoinorg/starcoin/blob/bf5ec6e44a242e9dff5ac177c1565c64c6e4b0d0/rpc/api/src/state/mod.rs#L14 etc
use bytes::Bytes;
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
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

use starcoin_rpc_schema_derive::rpc_schema;

#[rpc_schema]
#[rpc(client, server, namespace = "state2", namespace_separator = ".")]
pub trait StateApi {
    #[method(name = "get")]
    async fn get(&self, state_key: StateKey) -> RpcResult<Option<Bytes>>;

    /// Return state from StateTree storage directly by tree node key.
    #[method(name = "get_state_node_by_node_hash")]
    async fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> RpcResult<Option<Bytes>>;

    /// Return the Resource Or Code at the `access_path`, and provide a State Proof.
    #[method(name = "get_with_proof")]
    async fn get_with_proof(&self, state_key: StateKey) -> RpcResult<StateWithProofView>;

    /// Same as `state2.get_with_proof` but return `StateWithProof` in BCS serialize bytes.
    #[method(name = "get_with_proof_raw")]
    async fn get_with_proof_raw(&self, state_key: StateKey) -> RpcResult<StrView<Vec<u8>>>;

    #[method(name = "get_account_state")]
    async fn get_account_state(&self, address: AccountAddress) -> RpcResult<AccountState>;

    #[method(name = "get_account_state_set")]
    async fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> RpcResult<Option<AccountStateSetView>>;

    #[method(name = "get_state_root")]
    async fn get_state_root(&self) -> RpcResult<HashValue>;

    /// Return the Resource Or Code at the `access_path` and provide a State Proof at `state_root`
    #[method(name = "get_with_proof_by_root")]
    async fn get_with_proof_by_root(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> RpcResult<StateWithProofView>;

    /// Same as `state2.get_with_proof_by_root` but return `StateWithProof` in BCS serialize bytes.
    #[method(name = "get_with_proof_by_root_raw")]
    async fn get_with_proof_by_root_raw(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> RpcResult<StrView<Vec<u8>>>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    #[method(name = "get_with_table_item_proof")]
    async fn get_with_table_item_proof(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> RpcResult<StateWithTableItemProofView>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    #[method(name = "get_with_table_item_proof_by_root")]
    async fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> RpcResult<StateWithTableItemProofView>;

    /// get code of module
    #[method(name = "get_code")]
    async fn get_code(
        &self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> RpcResult<Option<CodeView>>;

    /// get resource data of `addr`
    #[method(name = "get_resource")]
    async fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> RpcResult<Option<ResourceView>>;

    /// list resources data of `addr`
    #[method(name = "list_resource")]
    async fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> RpcResult<ListResourceView>;

    /// list resources data of `addr`
    #[method(name = "list_code")]
    async fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> RpcResult<ListCodeView>;
}

pub use StateApiClient as StateApiRpcClient;
pub use StateApiServer as StateApiRpcServer;

/// Build jsonrpsee methods from legacy `StateApi`.
pub fn state_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: StateApiServer + Send + Sync + 'static,
{
    Ok(StateApiServer::into_rpc(api).into())
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
