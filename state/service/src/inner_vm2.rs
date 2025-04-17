use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::error;
use starcoin_state_api::{
    ChainStateReader, StateNodeStore, StateWithProof, StateWithTableItemProof,
};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::{
    account_state::AccountState, state_set::AccountStateSet, state_set::ChainStateSet,
};
use starcoin_vm2_statedb::{ChainStateDB as ChainStateDB2, ChainStateReader as ChainStateReader2};
use starcoin_vm_types::{
    access_path::AccessPath, account_address::AccountAddress,
    state_store::state_key::StateKey, state_store::table::TableHandle,
    state_store::table::TableInfo, state_view::StateView,
};
use std::sync::Arc;
use starcoin_types_glue::accounts::{AccountAddress2, HashValue1, HashValue2};

pub struct InnerVM2 {
    state_db: ChainStateDB2,
    //for adjust local time by on chain time.
    time_service: Arc<dyn TimeService>,
}

impl InnerVM2 {
    pub fn new(
        store: Arc<dyn StateNodeStore>,
        root_hash: Option<HashValue>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            state_db: ChainStateDB2::new(store, root_hash.map(|hash| HashValue2::from(hash).0)),
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
                    .get_account_state_set(&AccountAddress2::from(address)).map(|r| {
                    r.map(|state_set| {
                        AccountStateSet::new(state_set.0.clone())
                    })
                })
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
        let proof = reader.get_with_proof(&access_path_1_2(access_path))?;
        proof.into()
    }

    pub(crate) fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithTableItemProof> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_with_table_item_proof(&handle.into(), &key)?.into()
    }

    pub(crate) fn get_account_state_by_root(
        &self,
        account: AccountAddressVM1,
        state_root: HashValueVM1,
    ) -> anyhow::Result<Option<AccountState>> {
        let reader = self.state_db.fork_at(state_root);
        state_root.as_bytes()
        reader.get_account_state(&account)
    }

    pub(crate) fn change_root(&mut self, state_root: HashValueVM1) {
        self.state_db = self.state_db.fork_at(state_root);
        self.adjust_time();
    }

    pub fn adjust_time(&self) {
        match self.state_db.get_timestamp() {
            Ok(on_chain_time) => {
                self.time_service.adjust(on_chain_time.microseconds / 1000);
            }
            Err(e) => {
                error!("Get global time on chain fail: {:?}", e);
            }
        }
    }
}

impl ChainStateReader for InnerVM2 {
    fn get_with_proof(&self, access_path: &AccessPathVM1) -> anyhow::Result<StateWithProof> {
        self.state_db.get_with_proof(access_path)
    }

    fn get_account_state(
        &self,
        address: &AccountAddressVM1,
    ) -> anyhow::Result<Option<AccountState>> {
        self.state_db.get_account_state(address)
    }
    fn get_account_state_set(
        &self,
        address: &AccountAddressVM1,
    ) -> anyhow::Result<Option<AccountStateSet>> {
        self.state_db.get_account_state_set(address)
    }

    fn state_root(&self) -> HashValueVM1 {
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
        handle: &TableHandleVM1,
        key: &[u8],
    ) -> anyhow::Result<StateWithTableItemProof> {
        self.state_db.get_with_table_item_proof(handle, key)
    }

    fn get_table_info(&self, address: AccountAddressVM1) -> anyhow::Result<Option<TableInfo>> {
        self.state_db.get_table_info(address)
    }
}

impl StateView for InnerVM2 {
    fn get_state_value(&self, state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        self.state_db.get_state_value(state_key)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
