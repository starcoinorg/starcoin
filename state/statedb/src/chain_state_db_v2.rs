use crate::ChainStateDB;
use anyhow::format_err;
use bcs_ext::BCSCodec;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_state_api::{
    ChainStateReader, ChainStateWriter, StateNodeStore, StateWithProof, StateWithTableItemProof,
};
use starcoin_state_tree::StateTree;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::multi_state::MultiState;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::multi_vm_address;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::WriteSet;
use std::str::FromStr;
use std::sync::Arc;

/// multi vm nums
const MULTI_VM_NUMS: usize = 2;

static MULTI_STATE_PATH: Lazy<AccessPath> = Lazy::new(|| {
    let str = format!(
        "{}/1/{}::MultiState::MultiState",
        multi_vm_address(),
        multi_vm_address(),
    );
    AccessPath::from_str(str.as_str()).unwrap()
});
#[allow(clippy::upper_case_acronyms)]
pub struct ChainStateDBV2 {
    #[allow(dead_code)]
    store: Arc<dyn StateNodeStore>,
    chain_state_db: [ChainStateDB; MULTI_VM_NUMS],
}

#[allow(dead_code)]
impl ChainStateDBV2 {
    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        let global_state_tree: StateTree<AccountAddress> = StateTree::new(store.clone(), root_hash);
        let res = global_state_tree
            .get(&multi_vm_address())
            .expect("multi vm address should exist")
            .expect("should decode");
        let multi_state =
            bcs_ext::from_bytes::<MultiState>(res.as_slice()).expect("multi state should decode");
        let chain_state_db = [
            ChainStateDB::new(store.clone(), Some(*multi_state.state_root1())),
            ChainStateDB::new(store.clone(), Some(*multi_state.state_root2())),
        ];
        Self {
            store,
            chain_state_db,
        }
    }

    /// impl [`StateView::get_state_value`]
    fn get_state_value(&self, idx: usize, state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].get_state_value(state_key)
    }

    /// impl [`ChainStateReader::get_with_proof`]
    fn get_with_proof(
        &self,
        idx: usize,
        access_path: &AccessPath,
    ) -> anyhow::Result<StateWithProof> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].get_with_proof(access_path)
    }

    /// impl [`ChainStateReader::get_with_table_item_proof`]
    fn get_with_table_item_proof(
        &self,
        idx: usize,
        handle: &TableHandle,
        key: &[u8],
    ) -> anyhow::Result<StateWithTableItemProof> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].get_with_table_item_proof(handle, key)
    }

    /// impl [`ChainStateReader::get_table_info`]
    fn get_table_info(
        &self,
        idx: usize,
        address: AccountAddress,
    ) -> anyhow::Result<Option<TableInfo>> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].get_table_info(address)
    }

    /// impl [`ChainStateWriter::set`]
    fn set(&self, idx: usize, access_path: &AccessPath, value: Vec<u8>) -> anyhow::Result<()> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].set(access_path, value)
    }

    /// impl [`ChainStateWriter::remove`]
    fn remove(&self, idx: usize, access_path: &AccessPath) -> anyhow::Result<()> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].remove(access_path)
    }

    /// impl [`ChainStateWriter::apply`]

    fn apply(&self, idx: usize, chain_state_set: ChainStateSet) -> anyhow::Result<()> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].apply(chain_state_set)
    }

    /// impl [`ChainStateWriter::apply_write_set`]
    fn apply_write_set(&self, idx: usize, write_set: WriteSet) -> anyhow::Result<()> {
        if idx >= self.chain_state_db.len() {
            return Err(format_err!("index out of bounds: {}", idx));
        }
        self.chain_state_db[idx].apply_write_set(write_set)
    }

    /// impl [`ChainStateWriter::commit`]
    fn commit(&self) -> anyhow::Result<HashValue> {
        let state_root1 = self.chain_state_db[0].commit()?;
        let state_root2 = self.chain_state_db[1].commit()?;
        let multi_state = MultiState::new(state_root1, state_root2);
        self.chain_state_db[1].set(&MULTI_STATE_PATH, multi_state.encode()?)?;
        self.chain_state_db[1].commit()
    }

    /// impl [`ChainStateWriter::flush`]
    fn flush(&self) -> anyhow::Result<()> {
        self.chain_state_db[0].flush()?;
        self.chain_state_db[1].flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::chain_state_db_v2::MULTI_STATE_PATH;
    use crate::ChainStateDB;
    use bcs_ext::BCSCodec;
    use starcoin_crypto::HashValue;
    use starcoin_state_api::ChainStateWriter;
    use starcoin_state_tree::mock::MockStateNodeStore;
    use starcoin_types::multi_state::MultiState;
    use starcoin_vm_types::state_store::state_key::StateKey;
    use starcoin_vm_types::state_view::StateView;
    use std::sync::Arc;

    #[test]
    fn test_multi_state() {
        let state_root1 = HashValue::random();
        let state_root2 = HashValue::random();
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let multi_state = MultiState::new(state_root1, state_root2);
        chain_state_db
            .set(&MULTI_STATE_PATH, multi_state.encode().unwrap())
            .unwrap();
        chain_state_db.commit().unwrap();
        chain_state_db.flush().unwrap();
        let buf = chain_state_db
            .get_state_value(&StateKey::AccessPath(MULTI_STATE_PATH.clone()))
            .unwrap()
            .unwrap();
        let multi_state = MultiState::decode(&buf).unwrap();
        assert_eq!(*multi_state.state_root1(), state_root1);
        assert_eq!(*multi_state.state_root2(), state_root2);
    }
}
