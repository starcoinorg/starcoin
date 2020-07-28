// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{
    ChainStateReader, ChainStateService, StateNodeStore, StateView, StateWithProof,
};
use starcoin_statedb::ChainStateDB;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    state_set::ChainStateSet,
};
use std::sync::Arc;

pub struct ChainStateServiceImpl {
    //TODO use a StateReader
    reader: ChainStateDB,
}

impl ChainStateServiceImpl {
    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        Self {
            //TODO use a StateReader
            reader: ChainStateDB::new(store, root_hash),
        }
    }

    pub(crate) fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let reader = self.reader.change_root(state_root);
        reader.get_with_proof(&access_path)
    }
}

impl ChainStateService for ChainStateServiceImpl {
    fn change_root(&mut self, state_root: HashValue) {
        self.reader = self.reader.change_root(state_root);
    }
}

impl ChainStateReader for ChainStateServiceImpl {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.reader.get_with_proof(access_path)
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        self.reader.get_account_state(address)
    }

    fn state_root(&self) -> HashValue {
        self.reader.state_root()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }
}

impl StateView for ChainStateServiceImpl {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        self.reader.get(access_path)
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
