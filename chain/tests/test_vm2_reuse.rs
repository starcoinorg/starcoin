use anyhow::{anyhow, Result};
use starcoin_chain::{enable_vm2_reuse_for_test, ChainReader};
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_exec_merge::{global_witness_store, reset_global_witness_store_for_tests, ExecKey};
use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
use starcoin_types::block::Block;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_chain::{
    create_pre_state_fingerprint, reset_reuse_counters_for_test, reuse_counters_for_test,
};
use starcoin_vm2_crypto::HashValue as Vm2HashValue;
use starcoin_vm2_state_api::StateReaderExt;
use starcoin_vm2_test_helper::build_transfer_from_association;
use starcoin_vm2_types::transaction::{SignedUserTransaction, Transaction};
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

fn build_user_txn(chain: &MockChain, seq: u64, amount: u128) -> Result<SignedUserTransaction> {
    let txn = build_transfer_from_association(
        association_address(),
        seq,
        amount,
        chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        chain.net(),
    );
    match txn {
        Transaction::UserTransaction(txn) => Ok(txn),
        _ => Err(anyhow!("expected VM2 user transaction")),
    }
}

fn into_multi(txn: SignedUserTransaction) -> MultiSignedUserTransaction {
    MultiSignedUserTransaction::from(txn)
}

fn ghostdag_red_count(chain: &MockChain, block: &Block) -> Result<u64> {
    Ok(chain
        .head()
        .dag()
        .ghost_dag_manager()
        .ghostdag(block.header().parents_hash())?
        .mergeset_reds
        .len() as u64)
}

fn exec_key(tx_hash: Vm2HashValue, pre_state: Vm2HashValue) -> ExecKey {
    ExecKey {
        tx_hash,
        pre_state_fingerprint: pre_state,
    }
}

fn prepare_vm2_block(
    chain: &mut MockChain,
    txs: Vec<MultiSignedUserTransaction>,
) -> Result<(Block, HashValue, Vm2HashValue, Vec<Vm2HashValue>)> {
    let parent_block = chain.head().head_block();
    let parent_multi = parent_block.multi_state().clone();
    let parent_header = parent_block.header().clone();
    let parent_epoch = chain.head().epoch().number();
    let parent_header_id = parent_header.id();

    chain.net().time_service().sleep(1);
    let (template, excluded) = chain
        .head()
        .create_block_template_simple_with_txns(*chain.miner().address(), txs)?;
    assert!(
        excluded.discarded_txns.is_empty() && excluded.untouched_txns.is_empty(),
        "unexpected excluded txns: {:?}",
        excluded
    );

    let block = chain
        .head()
        .consensus()
        .create_block(template, chain.net().time_service().as_ref())?;
    let red_blocks = ghostdag_red_count(chain, &block)?;
    let metadata = block.to_metadata(parent_header.gas_used(), red_blocks);
    let pre_state =
        create_pre_state_fingerprint(parent_multi.state_root2(), &metadata, parent_epoch);
    let user_hashes = block
        .transactions2()
        .iter()
        .map(|txn| txn.id())
        .collect::<Vec<_>>();

    chain.apply(block.clone())?;

    Ok((block, parent_header_id, pre_state, user_hashes))
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
    let tx = into_multi(build_user_txn(&chain, seq, 1_000)?);

    let (block, parent_header_id, pre_state_fp, user_hashes) =
        prepare_vm2_block(&mut chain, vec![tx])?;
    assert!(!user_hashes.is_empty(), "expected at least one VM2 txn");

    // Ensure witness exists for the user transaction.
    let store = global_witness_store();
    for hash in &user_hashes {
        let key = exec_key(*hash, pre_state_fp);
        if let Some(mut rec) = store.get(&key) {
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
fn test_vm2_reuse_mixed_hit_and_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let tx0 = into_multi(build_user_txn(&chain, seq, 1_000)?);
    let tx1 = into_multi(build_user_txn(&chain, seq + 1, 2_000)?);

    let (block, parent_header_id, pre_state_fp, user_hashes) =
        prepare_vm2_block(&mut chain, vec![tx0, tx1])?;
    assert_eq!(user_hashes.len(), 2);

    let store = global_witness_store();
    // Preserve first witness, force second to re-execute.
    if let Some(mut rec) = store.get(&exec_key(user_hashes[0], pre_state_fp)) {
        rec.write_set
            .retain(|(k, _)| matches!(k.inner(), StateKeyInner::AccessPath(_)));
        assert!(rec.read_set.is_some(), "first txn witness missing read set");
        store.put(rec);
    } else {
        return Err(anyhow!("missing witness for first transaction"));
    }
    if let Some(mut rec) = store.get(&exec_key(user_hashes[1], pre_state_fp)) {
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
fn test_vm2_table_txn_triggers_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut chain = MockChain::new(ChainNetwork::new_test())?;
    let seq = chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let tx = into_multi(build_user_txn(&chain, seq, 5_000)?);

    let (block, parent_header_id, pre_state_fp, user_hashes) =
        prepare_vm2_block(&mut chain, vec![tx])?;
    assert_eq!(user_hashes.len(), 1);

    // Inject table write to disable reuse.
    let store = global_witness_store();
    let key = exec_key(user_hashes[0], pre_state_fp);
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
        hits <= total_vm2 - 1,
        "table write should prevent full reuse (hits={}, total={})",
        hits,
        total_vm2
    );

    Ok(())
}
