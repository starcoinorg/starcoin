// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::error;
use starcoin_state_api::{
    ChainStateReader, StateNodeStore, StateWithProof, StateWithTableItemProof,
};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_statedb::ChainStateDB;
use starcoin_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    state_set::{AccountStateSet, ChainStateSet},
};
use starcoin_vm_types::{
    access_path::AccessPath,
    state_store::{
        state_key::StateKey,
        table::{TableHandle, TableInfo},
    },
    state_view::{StateReaderExt, StateView},
};
use std::sync::Arc;

pub struct Inner {
    state_db: ChainStateDB,
    //for adjust local time by on chain time.
    time_service: Arc<dyn TimeService>,
}

impl Inner {
    pub fn new(
        store: Arc<dyn StateNodeStore>,
        root_hash: Option<HashValue>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            state_db: ChainStateDB::new(store, root_hash),
            time_service,
        }
    }

    pub(crate) fn get_account_state_set_with_root(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<AccountStateSet>> {
        match state_root {
            Some(root) => {
                let reader = self.state_db.fork_at(root);
                reader.get_account_state_set(&address)
            }
            None => self.get_account_state_set(&address),
        }
    }

    pub(crate) fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithProof> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_with_proof(&access_path)
    }

    pub(crate) fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithTableItemProof> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_with_table_item_proof(&handle, &key)
    }

    pub(crate) fn get_account_state_by_root(
        &self,
        account: AccountAddress,
        state_root: HashValue,
    ) -> anyhow::Result<Option<AccountState>> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_account_state(&account)
    }

    pub(crate) fn change_root(&mut self, state_root: HashValue) {
        self.state_db = self.state_db.fork_at(state_root);
        self.adjust_time();
    }

    pub fn adjust_time(&self) {
        match self.state_db.get_timestamp() {
            Ok(on_chain_time) => {
                self.time_service.adjust(on_chain_time.milliseconds);
            }
            Err(e) => {
                error!("Get global time on chain fail: {:?}", e);
            }
        }
    }
}

impl ChainStateReader for Inner {
    fn get_with_proof(&self, access_path: &AccessPath) -> anyhow::Result<StateWithProof> {
        self.state_db.get_with_proof(access_path)
    }

    fn get_account_state(&self, address: &AccountAddress) -> anyhow::Result<Option<AccountState>> {
        self.state_db.get_account_state(address)
    }
    fn get_account_state_set(
        &self,
        address: &AccountAddress,
    ) -> anyhow::Result<Option<AccountStateSet>> {
        self.state_db.get_account_state_set(address)
    }

    fn state_root(&self) -> HashValue {
        self.state_db.state_root()
    }

    fn dump(&self) -> anyhow::Result<ChainStateSet> {
        unimplemented!()
    }

    fn dump_iter(&self) -> anyhow::Result<AccountStateSetIterator> {
        unimplemented!()
    }

    fn get_with_table_item_proof(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> anyhow::Result<StateWithTableItemProof> {
        self.state_db.get_with_table_item_proof(handle, key)
    }

    fn get_table_info(&self, address: AccountAddress) -> anyhow::Result<Option<TableInfo>> {
        self.state_db.get_table_info(address)
    }
}

impl StateView for Inner {
    fn get_state_value(&self, state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        self.state_db.get_state_value(state_key)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
