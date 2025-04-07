// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as StateClient;
use crate::types::{
    AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView, TableInfoView,
};
use crate::FutureResult;
use openrpc_derive::openrpc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use starcoin_vm_types::state_store::table::TableHandle;
#[openrpc]
pub trait StateApi {
    #[rpc(name = "state.get")]
    fn get(&self, access_path: AccessPath) -> FutureResult<Option<Vec<u8>>>;

    /// Return state from StateTree storage directly by tree node key.
    #[rpc(name = "state.get_state_node_by_node_hash")]
    fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> FutureResult<Option<Vec<u8>>>;

    /// Return the Resource Or Code at the `access_path`, and provide a State Proof.
    #[rpc(name = "state.get_with_proof")]
    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProofView>;

    /// Same as `state.get_with_proof` but return `StateWithProof` in BCS serialize bytes.
    #[rpc(name = "state.get_with_proof_raw")]
    fn get_with_proof_raw(&self, access_path: AccessPath) -> FutureResult<StrView<Vec<u8>>>;

    #[rpc(name = "state.get_account_state")]
    fn get_account_state(&self, address: AccountAddress) -> FutureResult<Option<AccountState>>;

    #[rpc(name = "state.get_account_state_set")]
    fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> FutureResult<Option<AccountStateSetView>>;

    #[rpc(name = "state.get_state_root")]
    fn get_state_root(&self) -> FutureResult<HashValue>;

    /// Return the Resource Or Code at the `access_path` and provide a State Proof at `state_root`
    #[rpc(name = "state.get_with_proof_by_root")]
    fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StateWithProofView>;

    /// Same as `state.get_with_proof_by_root` but return `StateWithProof` in BCS serialize bytes.
    #[rpc(name = "state.get_with_proof_by_root_raw")]
    fn get_with_proof_by_root_raw(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StrView<Vec<u8>>>;

    /// Return the TableInfo according to queried AccountAddress
    #[rpc(name = "state.get_table_info")]
    fn get_table_info(&self, address: AccountAddress) -> FutureResult<Option<TableInfoView>>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    #[rpc(name = "state.get_with_table_item_proof")]
    fn get_with_table_item_proof(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> FutureResult<StateWithTableItemProofView>;

    /// Return the TableItem value  and provide a State Proof at `state_root`
    #[rpc(name = "state.get_with_table_item_proof_by_root")]
    fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> FutureResult<StateWithTableItemProofView>;

    /// get code of module
    #[rpc(name = "state.get_code")]
    fn get_code(
        &self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> FutureResult<Option<CodeView>>;

    /// get resource data of `addr`
    #[rpc(name = "state.get_resource")]
    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> FutureResult<Option<ResourceView>>;

    /// list resources data of `addr`
    #[rpc(name = "state.list_resource")]
    fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> FutureResult<ListResourceView>;

    /// list resources data of `addr`
    #[rpc(name = "state.list_code")]
    fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> FutureResult<ListCodeView>;
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
        ListResourceOption {
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
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
