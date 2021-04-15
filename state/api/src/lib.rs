// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{StateRequest, StateResponse};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

pub use chain_state::{
    AccountStateReader, ChainState, ChainStateReader, ChainStateWriter, StateProof, StateReaderExt,
    StateWithProof,
};
use serde::de::DeserializeOwned;
pub use starcoin_state_tree::StateNodeStore;
use starcoin_types::state_set::AccountStateSet;
use starcoin_vm_types::move_resource::MoveResource;
pub use starcoin_vm_types::state_view::StateView;

mod chain_state;
pub mod message;
pub mod mock;

#[async_trait::async_trait]
pub trait ChainStateAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>>;

    async fn get_with_proof(self, access_path: AccessPath) -> Result<StateWithProof>;

    async fn get_resource<R>(self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let access_path = AccessPath::new(address, R::resource_path());
        let r = self.get(access_path).await.and_then(|state| match state {
            Some(state) => Ok(Some(bcs_ext::from_bytes::<R>(state.as_slice())?)),
            None => Ok(None),
        })?;
        Ok(r)
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<Option<AccountState>>;

    /// get account stateset on state_root(if empty, use current state root).
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<Option<AccountStateSet>>;

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

#[async_trait::async_trait]
impl<S> ChainStateAsyncService for ServiceRef<S>
where
    S: ActorService + ServiceHandler<S, StateRequest>,
{
    async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>> {
        let response = self.send(StateRequest::Get(access_path)).await??;
        if let StateResponse::State(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof(self, access_path: AccessPath) -> Result<StateWithProof> {
        let response = self.send(StateRequest::GetWithProof(access_path)).await??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<Option<AccountState>> {
        let response = self.send(StateRequest::GetAccountState(address)).await??;
        if let StateResponse::AccountState(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<Option<AccountStateSet>> {
        let response = self
            .send(StateRequest::GetAccountStateSet {
                address,
                state_root,
            })
            .await??;
        if let StateResponse::AccountStateSet(state) = response {
            Ok(state)
        } else {
            panic!("Unexpected response type.")
        }
    }
    async fn state_root(self) -> Result<HashValue> {
        let response = self.send(StateRequest::StateRoot()).await??;
        if let StateResponse::StateRoot(root) = response {
            Ok(root)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let response = self
            .send(StateRequest::GetWithProofByRoot(access_path, state_root))
            .await??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state_by_root(
        self,
        account_address: AccountAddress,
        state_root: HashValue,
    ) -> Result<Option<AccountState>> {
        let response = self
            .send(StateRequest::GetAccountStateByRoot(
                account_address,
                state_root,
            ))
            .await??;
        if let StateResponse::AccountState(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }
}
