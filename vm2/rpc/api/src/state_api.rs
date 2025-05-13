// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as StateClient;
use crate::FutureResult;
// copy from https://github.com/starcoinorg/starcoin/blob/bf5ec6e44a242e9dff5ac177c1565c64c6e4b0d0/rpc/api/src/state/mod.rs#L14 etc
use bytes::Bytes;
use openrpc_derive::openrpc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_types::{
    view::{
        AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
        StateWithProofView, StateWithTableItemProofView, StrView, StructTagView, TableInfoView,
    },
    {account_address::AccountAddress, account_state::AccountState},
};
use starcoin_vm2_vm_types::{
    language_storage::{ModuleId, StructTag},
    state_store::{state_key::StateKey, table::TableHandle},
};
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

    /// Return the TableInfo according to queried AccountAddress
    #[rpc(name = "state2.get_table_info")]
    fn get_table_info(&self, address: AccountAddress) -> FutureResult<TableInfoView>;

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
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
