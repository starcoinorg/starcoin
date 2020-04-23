// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_crypto::HashValue;
use starcoin_state_api::StateWithProof;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

pub use self::gen_client::Client as StateClient;

#[rpc]
pub trait StateApi {
    #[rpc(name = "state.get")]
    fn get(&self, access_path: AccessPath) -> FutureResult<Option<Vec<u8>>>;

    #[rpc(name = "state.get_with_proof")]
    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProof>;

    #[rpc(name = "state.get_account_state")]
    fn get_account_state(&self, address: AccountAddress) -> FutureResult<Option<AccountState>>;

    #[rpc(name = "state.get_state_root")]
    fn get_state_root(&self) -> FutureResult<HashValue>;
}
