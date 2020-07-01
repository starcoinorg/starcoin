// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::RpcClient;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateView, StateWithProof};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::ChainStateSet;

pub struct RemoteStateReader<'a> {
    //TODO add cache.
    client: &'a RpcClient,
}

impl<'a> RemoteStateReader<'a> {
    pub fn new(client: &'a RpcClient) -> Self {
        Self { client }
    }
}

impl<'a> ChainStateReader for RemoteStateReader<'a> {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.client.state_get_with_proof(access_path.clone())
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        self.client.state_get_account_state(*address)
    }

    fn state_root(&self) -> HashValue {
        //TODO change trait api to return Result<HashValue>
        self.client
            .state_get_state_root()
            .expect("unexpected error.")
    }

    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }
}

impl<'a> StateView for RemoteStateReader<'a> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        self.client.state_get(access_path.clone())
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
