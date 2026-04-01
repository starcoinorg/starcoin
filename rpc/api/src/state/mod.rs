// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::{
    AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView,
};
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use starcoin_vm_types::state_store::table::TableHandle;

#[rpc(client, server, namespace = "state", namespace_separator = ".")]
pub trait StateApi {
    #[method(name = "get")]
    async fn get(&self, access_path: AccessPath) -> RpcResult<Option<Vec<u8>>>;

    /// Return state from StateTree storage directly by tree node key.
    #[method(name = "get_state_node_by_node_hash")]
    async fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> RpcResult<Option<Vec<u8>>>;

    /// Return the Resource Or Code at the `access_path`, and provide a State Proof.
    #[method(name = "get_with_proof")]
    async fn get_with_proof(&self, access_path: AccessPath) -> RpcResult<StateWithProofView>;

    /// Same as `state.get_with_proof` but return `StateWithProof` in BCS serialize bytes.
    #[method(name = "get_with_proof_raw")]
    async fn get_with_proof_raw(&self, access_path: AccessPath) -> RpcResult<StrView<Vec<u8>>>;

    #[method(name = "get_account_state")]
    async fn get_account_state(&self, address: AccountAddress) -> RpcResult<Option<AccountState>>;

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
        access_path: AccessPath,
        state_root: HashValue,
    ) -> RpcResult<StateWithProofView>;

    /// Same as `state.get_with_proof_by_root` but return `StateWithProof` in BCS serialize bytes.
    #[method(name = "get_with_proof_by_root_raw")]
    async fn get_with_proof_by_root_raw(
        &self,
        access_path: AccessPath,
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
