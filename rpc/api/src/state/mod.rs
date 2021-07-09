// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as StateClient;
use crate::types::{AccountStateSetView, CodeView, ResourceView, StateWithProofView, StrView};
use crate::FutureResult;
use jsonrpc_derive::rpc;
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

#[rpc]
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
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(default)]
pub struct ListResourceOption {
    pub decode: bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(default)]
pub struct GetResourceOption {
    pub decode: bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(default)]
pub struct GetCodeOption {
    pub resolve: bool,
}
