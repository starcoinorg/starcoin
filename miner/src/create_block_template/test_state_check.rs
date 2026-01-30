use super::state_check::StateCheck;
use anyhow::{format_err, Result};
use starcoin_state_api::{
    AccountStateSetIterator, ChainStateReader as ChainStateReader1, StateView as StateView1,
    StateWithProof, StateWithTableItemProof,
};
use starcoin_types::{
    access_path::AccessPath as AccessPath1,
    account::peer_to_peer_txn as peer_to_peer_txn1,
    account::Account as Account1,
    account::DEFAULT_EXPIRATION_TIME as DEFAULT_EXPIRATION_TIME1,
    account_address::AccountAddress,
    account_state::AccountState,
    event::EventHandle as EventHandle1,
    multi_transaction::MultiSignedUserTransaction,
    state_set::{AccountStateSet, ChainStateSet},
    transaction::SignedUserTransaction as SignedUserTransaction1,
};
use starcoin_vm2_state_api::ChainStateReader as ChainStateReader2;
use starcoin_vm2_state_api::{
    StateWithProof as StateWithProof2, StateWithTableItemProof as StateWithTableItemProof2,
};
use starcoin_vm2_types::{
    account::peer_to_peer_txn as peer_to_peer_txn2, account::Account as Account2,
    account::DEFAULT_EXPIRATION_TIME as DEFAULT_EXPIRATION_TIME2,
    account_address::AccountAddress as AccountAddress2,
    account_state::AccountState as AccountState2,
};
use starcoin_vm2_vm_types::{
    access_path::DataPath as DataPath2, account_config::AccountResource as AccountResource2,
    event::EventKey as EventKey2, move_resource::MoveStructType,
    on_chain_resource::ChainId as ChainId2, state_store::state_key::inner::StateKeyInner,
    state_store::state_key::StateKey as StateKey2,
    state_store::state_storage_usage::StateStorageUsage, state_store::TStateView,
};
use starcoin_vm_types::{
    account_config::AccountResource as AccountResource1, genesis_config::ChainId as ChainId1,
    move_resource::MoveResource, state_store::state_key::StateKey as StateKey1,
    state_store::table::TableHandle as TableHandle1,
};
use std::collections::HashMap;

struct TestStateReader1 {
    seqs: HashMap<AccountAddress, u64>,
}

impl TestStateReader1 {
    fn new(seqs: HashMap<AccountAddress, u64>) -> Self {
        Self { seqs }
    }
}

impl StateView1 for TestStateReader1 {
    fn get_state_value(&self, state_key: &StateKey1) -> Result<Option<Vec<u8>>> {
        let StateKey1::AccessPath(access_path) = state_key else {
            return Ok(None);
        };
        if access_path.path != AccountResource1::resource_path() {
            return Ok(None);
        }
        let Some(seq) = self.seqs.get(&access_path.address) else {
            return Ok(None);
        };
        let handle = EventHandle1::random_handle(0);
        let resource = AccountResource1::new(
            *seq,
            AccountResource1::DUMMY_AUTH_KEY.to_vec(),
            None,
            None,
            handle.clone(),
            handle.clone(),
            handle,
        );
        Ok(Some(bcs_ext::to_bytes(&resource)?))
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

impl ChainStateReader1 for TestStateReader1 {
    fn get_with_proof(&self, _access_path: &AccessPath1) -> Result<StateWithProof> {
        Err(format_err!("not used in test"))
    }

    fn get_account_state(&self, _address: &AccountAddress) -> Result<Option<AccountState>> {
        Ok(None)
    }

    fn get_account_state_set(&self, _address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        Ok(None)
    }

    fn state_root(&self) -> starcoin_crypto::HashValue {
        starcoin_crypto::HashValue::zero()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        Err(format_err!("not used in test"))
    }

    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        Err(format_err!("not used in test"))
    }

    fn get_with_table_item_proof(
        &self,
        _handle: &TableHandle1,
        _key: &[u8],
    ) -> Result<StateWithTableItemProof> {
        Err(format_err!("not used in test"))
    }
}

struct TestStateReader2 {
    seqs: HashMap<AccountAddress2, u64>,
}

impl TestStateReader2 {
    fn new(seqs: HashMap<AccountAddress2, u64>) -> Self {
        Self { seqs }
    }
}

impl TStateView for TestStateReader2 {
    type Key = StateKey2;

    fn get_state_value(
        &self,
        state_key: &Self::Key,
    ) -> starcoin_vm2_vm_types::state_store::Result<
        Option<starcoin_vm2_vm_types::state_store::state_value::StateValue>,
    > {
        let StateKeyInner::AccessPath(access_path) = state_key.inner() else {
            return Ok(None);
        };
        let DataPath2::Resource(struct_tag) = &access_path.path else {
            return Ok(None);
        };
        if struct_tag != &AccountResource2::struct_tag() {
            return Ok(None);
        }
        let Some(seq) = self.seqs.get(&access_path.address) else {
            return Ok(None);
        };
        let handle = starcoin_vm2_vm_types::event::EventHandle::new(
            EventKey2::new(0, AccountAddress2::ZERO),
            0,
        );
        let resource = AccountResource2::new(
            *seq,
            AccountResource2::DUMMY_AUTH_KEY.to_vec(),
            handle.clone(),
            handle,
        );
        Ok(Some(bcs_ext::to_bytes(&resource)?.into()))
    }

    fn get_usage(&self) -> starcoin_vm2_vm_types::state_store::Result<StateStorageUsage> {
        Ok(StateStorageUsage::zero())
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

impl ChainStateReader2 for TestStateReader2 {
    fn get_with_proof(&self, _state_key: &StateKey2) -> Result<StateWithProof2> {
        Err(format_err!("not used in test"))
    }

    fn get_account_state(&self, _address: &AccountAddress2) -> Result<AccountState2> {
        Err(format_err!("not used in test"))
    }

    fn get_account_state_set(
        &self,
        _address: &AccountAddress2,
    ) -> Result<Option<starcoin_vm2_types::state_set::AccountStateSet>> {
        Ok(None)
    }

    fn state_root(&self) -> starcoin_crypto::HashValue {
        starcoin_crypto::HashValue::zero()
    }

    fn dump(&self) -> Result<starcoin_vm2_types::state_set::ChainStateSet> {
        Err(format_err!("not used in test"))
    }

    fn dump_iter(&self) -> Result<starcoin_vm2_state_api::AccountStateSetIterator> {
        Err(format_err!("not used in test"))
    }

    fn get_with_table_item_proof(
        &self,
        _handle: &starcoin_vm2_vm_types::state_store::table::TableHandle,
        _key: &[u8],
    ) -> Result<StateWithTableItemProof2> {
        Err(format_err!("not used in test"))
    }
}

fn make_vm1_txn(sender: &Account1, seq: u64) -> SignedUserTransaction1 {
    let receiver = Account1::new();
    peer_to_peer_txn1(
        sender,
        &receiver,
        seq,
        1,
        DEFAULT_EXPIRATION_TIME1,
        ChainId1::test(),
    )
}

fn make_vm2_txn(
    sender: &Account2,
    seq: u64,
) -> starcoin_vm2_vm_types::transaction::SignedUserTransaction {
    let receiver = Account2::new();
    peer_to_peer_txn2(
        sender,
        &receiver,
        seq,
        1,
        DEFAULT_EXPIRATION_TIME2,
        ChainId2::test(),
    )
}

fn sort_transactions(transactions: &mut [MultiSignedUserTransaction]) {
    transactions.sort_by(|a, b| match a.sender().to_hex().cmp(&b.sender().to_hex()) {
        std::cmp::Ordering::Equal => a.sequence_number().cmp(&b.sequence_number()),
        other => other,
    });
}

#[test]
fn filter_continuous_transactions_rejects_gaps() -> Result<()> {
    let sender1 = Account1::new();
    let sender2 = Account2::new();

    let mut seqs1 = HashMap::new();
    seqs1.insert(*sender1.address(), 1);
    let mut seqs2 = HashMap::new();
    seqs2.insert(*sender2.address(), 10);

    let state_reader1 = TestStateReader1::new(seqs1);
    let state_reader2 = TestStateReader2::new(seqs2);
    let mut state_check = StateCheck::new(&state_reader1, &state_reader2);

    let mut transactions = vec![
        MultiSignedUserTransaction::VM1(make_vm1_txn(&sender1, 1)),
        MultiSignedUserTransaction::VM1(make_vm1_txn(&sender1, 3)),
        MultiSignedUserTransaction::VM2(make_vm2_txn(&sender2, 11)),
        MultiSignedUserTransaction::VM2(make_vm2_txn(&sender2, 10)),
    ];
    sort_transactions(&mut transactions);

    let filtered = state_check.filter_continuous_transactions(transactions)?;
    let vm1_seqs: Vec<u64> = filtered
        .iter()
        .filter_map(|txn| match txn {
            MultiSignedUserTransaction::VM1(txn) => Some(txn.sequence_number()),
            _ => None,
        })
        .collect();
    let vm2_seqs: Vec<u64> = filtered
        .iter()
        .filter_map(|txn| match txn {
            MultiSignedUserTransaction::VM2(txn) => Some(txn.sequence_number()),
            _ => None,
        })
        .collect();

    assert_eq!(vm1_seqs, vec![1]);
    assert_eq!(vm2_seqs, vec![10, 11]);
    Ok(())
}

#[test]
fn filter_continuous_transactions_uses_cache() -> Result<()> {
    let sender1 = Account1::new();

    let mut seqs1 = HashMap::new();
    seqs1.insert(*sender1.address(), 5);
    let state_reader1 = TestStateReader1::new(seqs1);
    let state_reader2 = TestStateReader2::new(HashMap::new());
    let mut state_check = StateCheck::new(&state_reader1, &state_reader2);

    let first = vec![MultiSignedUserTransaction::VM1(make_vm1_txn(&sender1, 5))];
    let second = vec![MultiSignedUserTransaction::VM1(make_vm1_txn(&sender1, 6))];

    let filtered_first = state_check.filter_continuous_transactions(first)?;
    let filtered_second = state_check.filter_continuous_transactions(second)?;

    assert_eq!(filtered_first.len(), 1);
    assert_eq!(filtered_second.len(), 1);
    Ok(())
}
