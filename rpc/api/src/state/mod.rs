// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as StateClient;
use crate::types::{
    AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
    StateWithProofView, StrView,
};
use crate::FutureResult;
use jsonrpc_derive::rpc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
#[rpc(client, server, schema)]
pub trait StateApi {
    #[rpc(name = "state.get")]
    fn get(&self, access_path: AccessPath) -> FutureResult<Option<Vec<u8>>>;

    #[rpc(name = "state.get_with_proof")]
    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProofView>;

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

    #[rpc(name = "state.get_with_proof_by_root")]
    fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StateWithProofView>;

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

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct ListResourceOption {
    pub decode: bool,
    /// The state tree root, default is the latest block state root
    pub state_root: Option<HashValue>,
    //TODO support filter by type and pagination
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
    let schema = rpc_impl_StateApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
