use anyhow::{anyhow, Result};
use starcoin_chain::{disable_vm2_reuse_for_test, enable_vm2_reuse_for_test, ChainReader};
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_exec_merge::{
    global_witness_store, reset_global_witness_store_for_tests, ExecKey, ReadEntry,
};
use starcoin_logger::prelude::*;
use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_chain::{reset_reuse_counters_for_test, reuse_counters_for_test};
use starcoin_vm2_state_api::StateReaderExt;
use starcoin_vm2_test_helper::build_transfer_from_association;
use starcoin_vm2_types::transaction::Transaction;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress as AccountAddress2,
    account_config::association_address,
    language_storage::TypeTag,
    state_store::{
        state_key::{inner::StateKeyInner, StateKey},
        table::{TableHandle, TableInfo},
    },
    write_set::WriteOp,
};

const TABLE_MARKER_KEY: &[u8] = b"reuse-table-marker";

#[test]
fn test_vm2_reuse_missing_reads_force_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let txn = match build_transfer_from_association(
        association_address(),
        seq,
        1_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };

    let parent_block = chain.head().head_block();
    let parent_header = parent_block.header().clone();
    let parent_epoch = chain.head().epoch().number();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let block = chain.produce_and_apply_by_tips_with_txns(
        parent_header.clone(),
        vec![parent_header_id],
        vec![MultiSignedUserTransaction::from(txn)],
    )?;

    let epoch_id = parent_epoch;
    let user_hashes = block
        .transactions2()
        .iter()
        .map(|txn| txn.id())
        .collect::<Vec<_>>();
    assert!(!user_hashes.is_empty(), "expected at least one VM2 txn");

    // Wipe read sets to simulate incomplete witness coverage.
    let store = global_witness_store();
    for hash in &user_hashes {
        let key = ExecKey {
            tx_hash: *hash,
            epoch_id,
        };
        if let Some(mut rec) = store.get(&key) {
            rec.read_set = Some(Vec::<ReadEntry>::new());
            store.put(rec);
        } else {
            panic!("missing witness for txn {:?}", hash);
        }
    }

    reset_reuse_counters_for_test();
    let mut fork = chain.fork(Some(parent_header_id))?;
    fork.apply(block.clone())?;
    let (hits, reexec) = reuse_counters_for_test();
    assert_eq!(hits, 0, "missing reads must force reexec");
    assert!(
        reexec > 0,
        "expected reexecution when witness coverage is incomplete"
    );

    let fork_root = fork.head().head_block().multi_state().state_root2();

    reset_reuse_counters_for_test();
    let mut full_branch = chain.fork(Some(parent_header_id))?;
    full_branch.apply(block)?;
    let full_root = full_branch.head().head_block().multi_state().state_root2();
    assert_eq!(
        fork_root, full_root,
        "state roots must match even when witness reuse is disabled"
    );

    Ok(())
}

#[test]
fn test_vm2_reuse_hits() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let txn = match build_transfer_from_association(
        association_address(),
        seq,
        1_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };

    let parent_block = chain.head().head_block();
    let parent_header = parent_block.header().clone();
    let parent_epoch = chain.head().epoch().number();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let block = chain.produce_and_apply_by_tips_with_txns(
        parent_header.clone(),
        vec![parent_header_id],
        vec![MultiSignedUserTransaction::from(txn)],
    )?;

    let epoch_id = parent_epoch;
    let user_hashes = block
        .transactions2()
        .iter()
        .map(|txn| txn.id())
        .collect::<Vec<_>>();
    assert!(!user_hashes.is_empty(), "expected at least one VM2 txn");

    // Ensure witness exists for the user transaction.
    let store = global_witness_store();
    for hash in &user_hashes {
        let key = ExecKey {
            tx_hash: *hash,
            epoch_id,
        };
        if let Some(mut rec) = store.get(&key) {
            info!(
                "[VM2 reuse test] witness tx={:?} reads={} writes={}",
                hash,
                rec.read_set.as_ref().map(|reads| reads.len()).unwrap_or(0),
                rec.write_set.len()
            );
            for (idx, (key, _)) in rec.write_set.iter().enumerate() {
                info!("[VM2 reuse test]   write[{idx}] key={:?}", key);
            }
            let had_reads = rec.read_set.is_some();
            rec.write_set
                .retain(|(k, _)| matches!(k.inner(), StateKeyInner::AccessPath(_)));
            assert!(had_reads, "witness missing read set for txn {:?}", hash);
            store.put(rec);
        } else {
            panic!("missing witness for txn {:?}", hash);
        }
    }

    reset_reuse_counters_for_test();

    let mut fork_chain = chain.fork(Some(parent_header_id))?;
    fork_chain.apply(block.clone())?;

    let (hits, reexec) = reuse_counters_for_test();
    assert!(
        hits >= user_hashes.len(),
        "expected reuse for user txns, hits={}",
        hits
    );
    assert!(
        reexec <= 2,
        "metadata/epilogue re-executions should be limited, got {}",
        reexec
    );

    // Reapply on another fork to confirm caching works twice.
    reset_reuse_counters_for_test();
    let mut second_fork = chain.fork(Some(parent_header_id))?;
    second_fork.apply(block)?;
    let (hits_again, _) = reuse_counters_for_test();
    assert!(
        hits_again >= user_hashes.len(),
        "expected consistent reuse on repeated forks"
    );

    Ok(())
}

#[test]
fn test_vm2_reuse_disabled_state_root_matches() -> Result<()> {
    let _guard = disable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let txn = match build_transfer_from_association(
        association_address(),
        seq,
        1_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };

    let parent_block = chain.head().head_block();
    let parent_header = parent_block.header().clone();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let block = chain.produce_and_apply_by_tips_with_txns(
        parent_header.clone(),
        vec![parent_header_id],
        vec![MultiSignedUserTransaction::from(txn)],
    )?;

    reset_reuse_counters_for_test();
    let mut disabled_branch = chain.fork(Some(parent_header_id))?;
    disabled_branch.apply(block.clone())?;
    let (hits, reexec) = reuse_counters_for_test();
    assert_eq!(hits, 0, "reuse should be disabled");
    assert!(reexec > 0, "expected transactions to re-execute");
    let reuse_disabled_root = disabled_branch
        .head()
        .head_block()
        .multi_state()
        .state_root2();

    reset_reuse_counters_for_test();
    let mut full_branch = chain.fork(Some(parent_header_id))?;
    full_branch.apply(block)?;
    let full_root = full_branch.head().head_block().multi_state().state_root2();

    assert_eq!(
        reuse_disabled_root, full_root,
        "state roots must match when reuse is disabled"
    );

    Ok(())
}

#[test]
fn test_vm2_reuse_mixed_hit_and_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let txn0 = match build_transfer_from_association(
        association_address(),
        seq,
        1_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };
    let txn1 = match build_transfer_from_association(
        association_address(),
        seq + 1,
        2_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };

    let parent_block = chain.head().head_block();
    let parent_header = parent_block.header().clone();
    let parent_epoch = chain.head().epoch().number();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let block = chain.produce_and_apply_by_tips_with_txns(
        parent_header.clone(),
        vec![parent_header_id],
        vec![
            MultiSignedUserTransaction::from(txn0),
            MultiSignedUserTransaction::from(txn1),
        ],
    )?;

    let epoch_id = parent_epoch;
    let user_hashes = block
        .transactions2()
        .iter()
        .map(|txn| txn.id())
        .collect::<Vec<_>>();
    assert_eq!(user_hashes.len(), 2);

    let store = global_witness_store();
    // Preserve first witness, force second to re-execute.
    if let Some(mut rec) = store.get(&ExecKey {
        tx_hash: user_hashes[0],
        epoch_id,
    }) {
        rec.write_set
            .retain(|(k, _)| matches!(k.inner(), StateKeyInner::AccessPath(_)));
        assert!(rec.read_set.is_some(), "first txn witness missing read set");
        store.put(rec);
    } else {
        return Err(anyhow!("missing witness for first transaction"));
    }
    if let Some(mut rec) = store.get(&ExecKey {
        tx_hash: user_hashes[1],
        epoch_id,
    }) {
        rec.read_set = None;
        store.put(rec);
    } else {
        return Err(anyhow!("missing witness for second transaction"));
    }

    reset_reuse_counters_for_test();

    let mut fork_chain = chain.fork(Some(parent_header_id))?;
    let block_clone = block.clone();
    fork_chain.apply(block)?;
    let (hits, reexec) = reuse_counters_for_test();
    assert!(
        hits >= 1,
        "expected reuse for at least one transaction, hits={}",
        hits
    );
    assert!(
        reexec >= 1,
        "expected forced re-execution when witness read_set cleared"
    );

    // Full re-execution path should yield identical state.
    reset_reuse_counters_for_test();
    let mut full_reexec = chain.fork(Some(parent_header_id))?;
    full_reexec.apply(block_clone)?;
    let reuse_root = fork_chain.head().head_block().multi_state().state_root2();
    let full_root = full_reexec.head().head_block().multi_state().state_root2();
    assert_eq!(reuse_root, full_root, "state roots must match");

    Ok(())
}

#[test]
fn test_vm2_reuse_conflicting_state_triggers_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let txn = match build_transfer_from_association(
        association_address(),
        seq,
        3_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };

    let parent_block = chain.head().head_block();
    let parent_header = parent_block.header().clone();
    let parent_epoch = chain.head().epoch().number();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let block = chain.produce_and_apply_by_tips_with_txns(
        parent_header.clone(),
        vec![parent_header_id],
        vec![MultiSignedUserTransaction::from(txn)],
    )?;

    let epoch_id = parent_epoch;

    // Ensure witnesses exist.
    let store = global_witness_store();
    for txn in block.transactions2().iter() {
        let hash = txn.id();
        let key = ExecKey {
            tx_hash: hash,
            epoch_id,
        };
        assert!(
            store.get(&key).is_some(),
            "missing witness for vm2 txn {:?}",
            hash
        );
    }

    // Clean branch: reuse should succeed.
    reset_reuse_counters_for_test();
    let mut clean_branch = chain.fork(Some(parent_header_id))?;
    clean_branch.apply(block.clone())?;
    let (clean_hits, clean_reexec) = reuse_counters_for_test();
    assert!(
        clean_hits >= 1 && clean_reexec <= 2,
        "expected reuse on clean branch (hits={}, reexec={})",
        clean_hits,
        clean_reexec
    );

    // Corrupt the witness to mimic a conflicting state (e.g., sequence already advanced).
    let mut corrupted = false;
    for txn in block.transactions2().iter() {
        let hash = txn.id();
        let key = ExecKey {
            tx_hash: hash,
            epoch_id,
        };
        if let Some(mut rec) = store.get(&key) {
            if let Some(reads) = rec.read_set.as_mut() {
                if let Some(entry) = reads
                    .iter_mut()
                    .find(|entry| format!("{:?}", entry.key).contains("account::Account"))
                {
                    entry.value_hash = HashValue::zero();
                    corrupted = true;
                }
            }
            store.put(rec);
        }
    }
    assert!(
        corrupted,
        "expected to corrupt account read entry in witness"
    );

    reset_reuse_counters_for_test();
    let mut conflicting_branch = chain.fork(Some(parent_header_id))?;
    conflicting_branch.apply(block.clone())?;
    let (conflict_hits, conflict_reexec) = reuse_counters_for_test();
    assert_eq!(
        conflict_hits, 0,
        "corrupted witness must force re-execution"
    );
    assert!(
        conflict_reexec >= 1,
        "expected re-execution path when conflicting read detected"
    );

    Ok(())
}

#[test]
fn test_vm2_table_txn_triggers_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let txn = match build_transfer_from_association(
        association_address(),
        seq,
        5_000,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    ) {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected VM2 user transaction")),
    };

    let parent_block = chain.head().head_block();
    let parent_header = parent_block.header().clone();
    let parent_epoch = chain.head().epoch().number();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let block = chain.produce_and_apply_by_tips_with_txns(
        parent_header.clone(),
        vec![parent_header_id],
        vec![MultiSignedUserTransaction::from(txn)],
    )?;

    let epoch_id = parent_epoch;
    let user_hashes = block
        .transactions2()
        .iter()
        .map(|txn| txn.id())
        .collect::<Vec<_>>();
    assert_eq!(user_hashes.len(), 1);

    // Inject table write to disable reuse.
    let store = global_witness_store();
    let key = ExecKey {
        tx_hash: user_hashes[0],
        epoch_id,
    };
    if let Some(mut rec) = store.get(&key) {
        let handle = TableHandle(AccountAddress2::new([0u8; 16]));
        let table_key = StateKey::table_item(&handle, TABLE_MARKER_KEY);
        rec.write_set.push((table_key, WriteOp::legacy_deletion()));
        rec.table_infos
            .push((handle, TableInfo::new(TypeTag::U64, TypeTag::U64)));
        store.put(rec);
    } else {
        return Err(anyhow!("missing witness for table gating test"));
    }

    reset_reuse_counters_for_test();

    let mut fork_chain = chain.fork(Some(parent_header_id))?;
    fork_chain.apply(block)?;

    let (hits, reexec) = reuse_counters_for_test();
    let total_vm2 = user_hashes.len() + 2; // metadata + user + epilogue
    assert_eq!(
        hits + reexec,
        total_vm2,
        "reuse counters should cover all vm2 txns (hits={}, reexec={}, total={})",
        hits,
        reexec,
        total_vm2
    );
    assert!(
        reexec >= 1,
        "table write should force at least one re-execution"
    );
    assert!(
        hits < total_vm2,
        "table write should prevent full reuse (hits={}, total={})",
        hits,
        total_vm2
    );

    Ok(())
}
