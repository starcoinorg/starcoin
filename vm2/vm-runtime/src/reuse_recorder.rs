use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::HashValue;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_value::StateValue;
use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct ReadDescriptor {
    pub key: StateKey,
    pub from_storage: bool,
    pub existed: bool,
    pub value_hash: HashValue,
}

#[derive(Default)]
struct Recorder {
    per_txn: Vec<Vec<ReadDescriptor>>,
    current: Vec<ReadDescriptor>,
    seen: HashSet<StateKey>,
}

thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

pub fn start() {
    RECORDER.with(|cell| {
        *cell.borrow_mut() = Some(Recorder {
            per_txn: Vec::new(),
            current: Vec::new(),
            seen: std::collections::HashSet::new(),
        });
    });
}

pub fn is_active() -> bool {
    RECORDER.with(|cell| cell.borrow().is_some())
}

pub fn reset_session() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            rec.current.clear();
            rec.per_txn.clear();
            rec.seen.clear();
        }
    });
}

pub fn begin_transaction() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            rec.current.clear();
            rec.seen.clear();
        }
    });
}

pub fn abort_transaction() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            rec.current.clear();
            rec.seen.clear();
        }
    });
}

pub fn end_transaction() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            let mut taken = Vec::new();
            std::mem::swap(&mut rec.current, &mut taken);
            rec.per_txn.push(taken);
        }
    });
}

pub fn record_read(state_key: &StateKey, from_storage: bool, value: Option<&StateValue>) {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            if !rec.seen.insert(state_key.clone()) {
                return;
            }
            let (existed, value_hash) = match value {
                Some(val) => (true, val.hash()),
                None => (false, HashValue::zero()),
            };

            rec.current.push(ReadDescriptor {
                key: state_key.clone(),
                from_storage,
                existed,
                value_hash,
            });
        }
    });
}

pub fn finish() -> Vec<Vec<ReadDescriptor>> {
    RECORDER.with(|cell| {
        cell.borrow_mut()
            .take()
            .map(|mut rec| {
                if !rec.current.is_empty() {
                    rec.per_txn.push(std::mem::take(&mut rec.current));
                }
                rec.per_txn
            })
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use starcoin_config::ChainNetwork;
    use starcoin_exec_merge::{self as exec_merge};
    use starcoin_genesis::vm2::{
        build_and_execute_genesis_transaction, execute_genesis_transaction,
    };
    use starcoin_transaction_builder::{
        vm2::build_transfer_from_association, DEFAULT_EXPIRATION_TIME,
    };
    use starcoin_vm2_executor::executor::do_execute_block_transactions;
    use starcoin_vm2_statedb::ChainStateDB;
    use starcoin_vm_types::{
        account_config::{association_address, AccountResource},
        state_store::state_key::StateKey,
        state_view::StateReaderExt,
        transaction::Transaction,
    };
    use std::collections::HashSet;

    #[test]
    fn recorder_captures_account_and_timestamp_reads() -> Result<()> {
        crate::starcoin_vm::StarcoinVM::set_concurrency_level_once(1);
        let net = ChainNetwork::new_test();
        let state = ChainStateDB::mock();

        let (genesis_txn, _) = build_and_execute_genesis_transaction(&net);
        execute_genesis_transaction(&state, Transaction::UserTransaction(genesis_txn))?;

        let seq = state
            .get_account_resource(association_address())?
            .sequence_number();
        let txn = build_transfer_from_association(
            association_address(),
            seq,
            1_000,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net.chain_id().id().into(),
            net.genesis_config2(),
        );
        let signed_txn = match txn {
            Transaction::UserTransaction(txn) => txn,
            _ => anyhow::bail!("expected VM2 user transaction"),
        };

        let account_key = StateKey::resource_typed::<AccountResource>(&association_address())?;

        let mut outputs = do_execute_block_transactions(
            &state,
            vec![Transaction::UserTransaction(signed_txn)],
            None,
            None,
        )?;
        assert_eq!(outputs.len(), 1);
        let output = outputs.pop().expect("vm output");
        let txn_status = output.status().clone();
        assert!(
            matches!(
                txn_status,
                starcoin_vm_types::transaction::TransactionStatus::Keep(_)
            ),
            "expected kept transaction, got {:?}",
            txn_status
        );
        let write_entries = output.write_set().clone().into_iter().collect::<Vec<_>>();

        let mut read_entries: Option<Vec<exec_merge::ReadEntry>> = None;
        exec_merge::hydrate_read_set_for_writes(&state, &mut read_entries, &write_entries)?;

        let entries = read_entries.expect("hydrated read set");
        for entry in entries.iter() {
            let current = state
                .get_state_value(&entry.key)
                .map_err(|e| anyhow::anyhow!("state read error: {:?}", e))?
                .map(|value| value.hash());
            assert_eq!(
                current,
                Some(entry.value_hash),
                "pre-state hash mismatch for key {:?}",
                entry.key
            );
        }

        let keys: HashSet<StateKey> = entries.into_iter().map(|entry| entry.key).collect();
        assert!(
            keys.contains(&account_key),
            "read set missing AccountResource key: {:?}",
            keys
        );
        assert!(
            !keys.is_empty(),
            "read set should not be empty after hydration"
        );

        Ok(())
    }
}
