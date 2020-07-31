// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_crypto::HashValue;

pub use starcoin_state_tree::StateNodeStore;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

mod chain_state;
pub mod mock;

pub use chain_state::{
    AccountStateReader, ChainState, ChainStateReader, ChainStateWriter, StateProof, StateWithProof,
};

pub use starcoin_vm_types::state_view::StateView;

pub trait ChainStateService: ChainStateReader {
    ///Use new state_root for load chain state.
    fn change_root(&mut self, state_root: HashValue);
}

#[async_trait::async_trait]
pub trait ChainStateAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>>;

    async fn get_with_proof(self, access_path: AccessPath) -> Result<StateWithProof>;

    async fn get_account_state(self, address: AccountAddress) -> Result<Option<AccountState>>;

    async fn state_root(self) -> Result<HashValue>;

    async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProof>;

    async fn get_account_state_by_root(
        self,
        address: AccountAddress,
        state_root: HashValue,
    ) -> Result<Option<AccountState>>;
}
