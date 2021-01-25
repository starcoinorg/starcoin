// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::RpcClient;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateView, StateWithProof};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};

pub struct RemoteStateReader<'a> {
    //TODO add cache.
    client: &'a RpcClient,
    state_root: HashValue,
}

impl<'a> RemoteStateReader<'a> {
    pub fn new(client: &'a RpcClient) -> Result<Self> {
        let state_root = client.state_get_state_root()?;
        Ok(Self::new_with_root(client, state_root))
    }

    pub fn new_with_root(client: &'a RpcClient, state_root: HashValue) -> Self {
        Self { client, state_root }
    }
}

impl<'a> ChainStateReader for RemoteStateReader<'a> {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.client
            .state_get_with_proof(access_path.clone())
            .map(Into::into)
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        self.client.state_get_account_state(*address)
    }

    fn state_root(&self) -> HashValue {
        //TODO change trait api to return Result<HashValue>
        self.state_root
    }

    fn get_account_state_set(&self, _address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        unimplemented!()
    }
    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }
}

impl<'a> StateView for RemoteStateReader<'a> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self
            .client
            .state_get_with_proof_by_root(access_path.clone(), self.state_root())?
            .state
            .map(|v| v.0))
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
