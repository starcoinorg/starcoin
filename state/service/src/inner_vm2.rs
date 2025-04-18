use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateWithProof, StateWithTableItemProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::{
    account_state::AccountState, state_set::AccountStateSet, state_set::ChainStateSet,
};
use starcoin_types_glue::{
    access_path, account_address, account_state, account_state_set, state_with_proof,
    state_with_table_item_proof, table_handle, table_info,
};
use starcoin_vm2_statedb::{ChainStateDB as ChainStateDB2, ChainStateReader as ChainStateReader2};
use starcoin_vm_types::{
    access_path::AccessPath, account_address::AccountAddress, state_store::state_key::StateKey,
    state_store::table::TableHandle, state_store::table::TableInfo, state_view::StateView,
};

use starcoin_vm2_state_api::StateNodeStore as StateNodeStoreVM2;
use std::sync::Arc;

pub struct InnerVM2 {
    state_db: ChainStateDB2,
    //for adjust local time by on chain time.
    time_service: Arc<dyn TimeService>,
}

impl InnerVM2 {
    pub fn new(
        store: Arc<dyn StateNodeStoreVM2>,
        root_hash: Option<HashValue>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            state_db: ChainStateDB2::new(store, root_hash),
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
                reader
                    .get_account_state_set(&account_address::vm1_to_vm2(address))
                    .map(|r| r.map(|state_set| account_state_set::vm2_to_vm1(state_set)))
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
        reader
            .get_with_proof(&access_path::vm1_to_vm2(access_path))
            .map(|proof| state_with_proof::vm2_to_vm1(proof))
    }

    pub(crate) fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithTableItemProof> {
        let reader = self.state_db.fork_at(state_root);
        reader
            .get_with_table_item_proof(&table_handle::vm1_to_vm2(handle), &key)
            .map(|proof| state_with_table_item_proof::vm2_to_vm1(proof))
    }

    pub(crate) fn get_account_state_by_root(
        &self,
        account: AccountAddress,
        state_root: HashValue,
    ) -> anyhow::Result<Option<AccountState>> {
        let reader = self.state_db.fork_at(state_root);
        reader
            .get_account_state(&account_address::vm1_to_vm2(account))
            .map(|s| Some(account_state::vm2_to_vm1(s)))
    }

    pub(crate) fn change_root(&mut self, state_root: HashValue) {
        self.state_db = self.state_db.fork_at(state_root);
        self.adjust_time();
    }

    pub fn adjust_time(&self) {
        // TODO(BobOng): [dual-vm] get_timestamp not implement for state_db, check and confirm it which layer to implement
        // match self.state_db.get_timestamp() {
        //     Ok(on_chain_time) => {
        //         self.time_service.adjust(on_chain_time.microseconds / 1000);
        //     }
        //     Err(e) => {
        //         error!("Get global time on chain fail: {:?}", e);
        //     }
        // }
    }
}

impl ChainStateReader for InnerVM2 {
    fn get_with_proof(&self, access_path: &AccessPath) -> anyhow::Result<StateWithProof> {
        self.state_db
            .get_with_proof(&access_path::vm1_to_vm2(access_path.clone()))
            .map(|proof| state_with_proof::vm2_to_vm1(proof))
    }

    fn get_account_state(&self, address: &AccountAddress) -> anyhow::Result<Option<AccountState>> {
        self.state_db
            .get_account_state(&account_address::vm1_to_vm2(address.clone()))
            .map(|stat| Some(account_state::vm2_to_vm1(stat)))
    }
    fn get_account_state_set(
        &self,
        address: &AccountAddress,
    ) -> anyhow::Result<Option<AccountStateSet>> {
        self.state_db
            .get_account_state_set(&account_address::vm1_to_vm2(address.clone()))
            .map(|state| {
                state.map(|account_state_set| account_state_set::vm2_to_vm1(account_state_set))
            })
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
        self.state_db
            .get_with_table_item_proof(&table_handle::vm1_to_vm2(handle.clone()), key)
            .map(|p| state_with_table_item_proof::vm2_to_vm1(p))
    }

    fn get_table_info(&self, address: AccountAddress) -> anyhow::Result<Option<TableInfo>> {
        self.state_db
            .get_table_info(account_address::vm1_to_vm2(address))
            .map(|table_info| Some(table_info::vm2_to_vm1(table_info)))
    }
}

impl StateView for InnerVM2 {
    fn get_state_value(&self, _state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        // self.state_db.get(state_key)
        // TODO(BobOng): [dual-vm] maybe not need to implements
        Ok(None)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
