use anyhow::{anyhow, Result};
use starcoin_chain::{enable_vm2_reuse_for_test, ChainReader};
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_exec_merge::{global_witness_store, reset_global_witness_store_for_tests, ExecKey};
use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_chain::{reset_reuse_counters_for_test, reuse_counters_for_test};
use starcoin_vm2_state_api::StateReaderExt;
use starcoin_vm2_test_helper::build_transfer_from_association;
use starcoin_vm2_types::transaction::{SignedUserTransaction, Transaction};
use starcoin_vm2_vm_types::account_config::association_address;

#[test]
fn test_vm2_reuse_hits_and_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;
    let parent_executed = mock_chain.head().head_block();
    let parent_multi = parent_executed.multi_state().clone();
    let parent_header = parent_executed.header().clone();
    let parent_epoch_number = mock_chain.head().epoch().number();

    let association_seq = mock_chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let receiver = association_address();
    let raw_txn = build_transfer_from_association(
        receiver,
        association_seq,
        1000,
        mock_chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        mock_chain.net(),
    );
    let signed: SignedUserTransaction = match raw_txn {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected vm2 user transaction")),
    };
    let multi = MultiSignedUserTransaction::from(signed);

    // Ensure block timestamp strictly increases in mock time.
    mock_chain.net().time_service().sleep(1);

    let (template, _) = mock_chain
        .head()
        .create_block_template_simple_with_txns(*mock_chain.miner().address(), vec![multi])?;
    let block = mock_chain
        .head()
        .consensus()
        .create_block(template, mock_chain.net().time_service().as_ref())?;
    let txn_count = block.transactions2().len();
    assert!(txn_count > 0, "block must contain VM2 transactions");

    let red_blocks = mock_chain
        .head()
        .dag()
        .ghost_dag_manager()
        .ghostdag(block.header().parents_hash())?
        .mergeset_reds
        .len() as u64;

    mock_chain.apply(block.clone())?;
    let metadata = block.to_metadata(parent_header.gas_used(), red_blocks);
    let pre_state_fp = starcoin_vm2_chain::create_pre_state_fingerprint(
        parent_multi.state_root2(),
        &metadata,
        parent_epoch_number,
    );
    let witness_store = global_witness_store();
    let user_tx_hash = block
        .transactions2()
        .first()
        .expect("block must contain vm2 txn")
        .id();
    let exec_key = ExecKey {
        tx_hash: user_tx_hash,
        pre_state_fingerprint: pre_state_fp,
    };
    assert!(
        witness_store.get(&exec_key).is_some(),
        "witness store should persist exec record for the first user transaction"
    );

    let planned_txns =
        starcoin_vm2_chain::build_block_transactions(block.transactions2(), Some(metadata.clone()));
    for tx in planned_txns.iter() {
        let key = ExecKey {
            tx_hash: tx.id(),
            pre_state_fingerprint: pre_state_fp,
        };
        assert!(
            witness_store.get(&key).is_some(),
            "witness store missing record for tx {:?}",
            tx.id()
        );
    }
    let total_txns = planned_txns.len();

    let (hits_after_first, reexec_after_first) = reuse_counters_for_test();
    assert!(
        reexec_after_first + hits_after_first > 0,
        "first execution should execute or reuse transactions"
    );

    let mut fork_chain = mock_chain.fork(Some(parent_header.id()))?;
    let fork_parent = fork_chain.head().head_block();
    let fork_parent_multi = fork_parent.multi_state().clone();
    let fork_red_blocks = fork_chain
        .head()
        .dag()
        .ghost_dag_manager()
        .ghostdag(block.header().parents_hash())?
        .mergeset_reds
        .len() as u64;
    let fork_metadata = block.to_metadata(parent_header.gas_used(), fork_red_blocks);
    let fork_pre_fp = starcoin_vm2_chain::create_pre_state_fingerprint(
        fork_parent_multi.state_root2(),
        &fork_metadata,
        fork_chain.head().epoch().number(),
    );
    assert_eq!(
        fork_pre_fp, pre_state_fp,
        "pre-state fingerprint mismatch between stored witness ({:?}) and fork execution ({:?})",
        pre_state_fp, fork_pre_fp
    );
    fork_chain.apply(block.clone())?;
    let (hits_after_second, reexec_after_second) = reuse_counters_for_test();
    let reuse_hits_delta = hits_after_second
        .checked_sub(hits_after_first)
        .expect("reuse hits should be monotonic");
    let reexec_delta = reexec_after_second
        .checked_sub(reexec_after_first)
        .expect("reexec counter should be monotonic");

    assert!(
        hits_after_second > hits_after_first,
        "second execution should report reuse hits: before={}, after={}",
        hits_after_first,
        hits_after_second
    );
    assert_eq!(
        reuse_hits_delta + reexec_delta,
        total_txns,
        "reuse+reexec delta ({}, {}) should equal total txns ({})",
        reuse_hits_delta,
        reexec_delta,
        total_txns
    );
    assert!(
        reuse_hits_delta >= txn_count,
        "expected to reuse at least {} user txns, but reuse delta was {} (hits before={}, after={})",
        txn_count,
        reuse_hits_delta,
        hits_after_first,
        hits_after_second
    );
    assert!(
        reexec_delta <= total_txns.saturating_sub(txn_count),
        "only system transactions should re-execute: total_first={}, user_txns={}, reexec_delta={}",
        total_txns,
        txn_count,
        reexec_delta
    );

    Ok(())
}

#[test]
fn test_vm2_reuse_mixed_hit_and_reexec() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;
    let parent_header = mock_chain.head().head_block().header().clone();
    let parent_multi = mock_chain.head().head_block().multi_state().clone();
    let parent_epoch_number = mock_chain.head().epoch().number();

    let association_seq = mock_chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;

    let receiver = association_address();
    let raw_txn_0 = build_transfer_from_association(
        receiver,
        association_seq,
        1000,
        mock_chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        mock_chain.net(),
    );
    let raw_txn_1 = build_transfer_from_association(
        receiver,
        association_seq + 1,
        2000,
        mock_chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        mock_chain.net(),
    );

    let signed_0: SignedUserTransaction = match raw_txn_0 {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected vm2 user transaction")),
    };
    let signed_1: SignedUserTransaction = match raw_txn_1 {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected vm2 user transaction")),
    };

    let multi_0 = MultiSignedUserTransaction::from(signed_0);
    let multi_1 = MultiSignedUserTransaction::from(signed_1);

    mock_chain.net().time_service().sleep(1);
    let (template, _) = mock_chain.head().create_block_template_simple_with_txns(
        *mock_chain.miner().address(),
        vec![multi_0, multi_1],
    )?;
    let block = mock_chain
        .head()
        .consensus()
        .create_block(template, mock_chain.net().time_service().as_ref())?;

    mock_chain.apply(block.clone())?;
    let first_state_root2 = mock_chain.head().head_block().multi_state().state_root2();

    let red_blocks = mock_chain
        .head()
        .dag()
        .ghost_dag_manager()
        .ghostdag(block.header().parents_hash())?
        .mergeset_reds
        .len() as u64;
    let metadata = block.to_metadata(parent_header.gas_used(), red_blocks);
    let pre_state_fp = starcoin_vm2_chain::create_pre_state_fingerprint(
        parent_multi.state_root2(),
        &metadata,
        parent_epoch_number,
    );
    let witness_store = global_witness_store();
    for tx in starcoin_vm2_chain::build_block_transactions(block.transactions2(), Some(metadata)) {
        let key = ExecKey {
            tx_hash: tx.id(),
            pre_state_fingerprint: pre_state_fp,
        };
        assert!(
            witness_store.get(&key).is_some(),
            "witness store missing record for tx {:?}",
            tx.id()
        );
    }

    reset_reuse_counters_for_test();
    let mut fork_chain = mock_chain.fork(Some(parent_header.id()))?;
    fork_chain.apply(block.clone())?;
    let (hits_after_second, reexec_after_second) = reuse_counters_for_test();
    assert!(
        hits_after_second > 0,
        "expected at least one transaction to reuse"
    );
    assert!(
        reexec_after_second > hits_after_second,
        "second run should trigger mixed reuse/reexec, got hits={}, reexec={}",
        hits_after_second,
        reexec_after_second
    );

    let second_state_root2 = fork_chain.head().head_block().multi_state().state_root2();
    assert_eq!(
        second_state_root2, first_state_root2,
        "state_root2 should be identical between full execution and selective reuse"
    );

    Ok(())
}

#[test]
fn test_vm2_reuse_multi_parent_block_execution() -> Result<()> {
    let _guard = enable_vm2_reuse_for_test();
    reset_reuse_counters_for_test();
    reset_global_witness_store_for_tests();

    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;
    let association_seq = mock_chain
        .head()
        .chain_state_reader2()
        .get_sequence_number(association_address())?;
    let mut next_seq = association_seq;
    let miner = *mock_chain.miner().address();

    let record_delta =
        |label: &str, before_hits: usize, before_reexec: usize| -> (String, usize, usize) {
            let (after_hits, after_reexec) = reuse_counters_for_test();
            (
                label.to_string(),
                after_hits - before_hits,
                after_reexec - before_reexec,
            )
        };

    // Block A extending genesis with one user txn.
    mock_chain.net().time_service().sleep(1);
    let block_a = {
        let raw = build_transfer_from_association(
            association_address(),
            next_seq,
            1_000,
            mock_chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            mock_chain.net(),
        );
        next_seq += 1;
        let signed = match raw {
            Transaction::UserTransaction(txn) => txn,
            _ => return Err(anyhow!("expected vm2 user transaction")),
        };
        let multi = MultiSignedUserTransaction::from(signed);
        let (template, _) = mock_chain
            .head()
            .create_block_template_simple_with_txns(miner, vec![multi])?;
        mock_chain
            .head()
            .consensus()
            .create_block(template, mock_chain.net().time_service().as_ref())?
    };
    let (hits_before_a, reexec_before_a) = reuse_counters_for_test();
    mock_chain.apply(block_a.clone())?;
    let _ = record_delta("A", hits_before_a, reexec_before_a);

    let mut parent_header = block_a.header().clone();

    // Produce linear blocks B1..B5 (each extends previous) with distinct txns.
    let mut lineage_ids = Vec::new();
    for idx in 0..5u64 {
        mock_chain.net().time_service().sleep(1);
        let raw = build_transfer_from_association(
            association_address(),
            next_seq,
            10_000 + idx as u128,
            mock_chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            mock_chain.net(),
        );
        next_seq += 1;
        let signed = match raw {
            Transaction::UserTransaction(txn) => txn,
            _ => return Err(anyhow!("expected vm2 user transaction")),
        };
        let multi = MultiSignedUserTransaction::from(signed);
        let tips = vec![parent_header.id()];
        let (template, _) = mock_chain.head().create_block_template(
            miner,
            Some(parent_header.clone()),
            vec![multi],
            None,
            None,
            Some(tips),
            HashValue::zero(),
        )?;
        let block = mock_chain
            .head()
            .consensus()
            .create_block(template, mock_chain.net().time_service().as_ref())?;
        let (hits_before, reexec_before) = reuse_counters_for_test();
        mock_chain.apply(block.clone())?;
        let stats = record_delta(&format!("B{}", idx + 1), hits_before, reexec_before);
        assert!(
            stats.2 > 0,
            "expected block {} to execute new transactions",
            stats.0
        );
        lineage_ids.push(block.id());
        parent_header = block.header().clone();
    }

    // Produce block C referencing B1..B5 and record deltas.
    mock_chain.net().time_service().sleep(1);
    let raw_c = build_transfer_from_association(
        association_address(),
        next_seq,
        99_999,
        mock_chain.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        mock_chain.net(),
    );
    let signed_c = match raw_c {
        Transaction::UserTransaction(txn) => txn,
        _ => return Err(anyhow!("expected vm2 user transaction")),
    };
    let multi_c = MultiSignedUserTransaction::from(signed_c);
    let (template_c, _) = mock_chain.head().create_block_template(
        miner,
        None,
        vec![multi_c],
        None,
        None,
        Some(lineage_ids),
        HashValue::zero(),
    )?;
    let block_c = mock_chain
        .head()
        .consensus()
        .create_block(template_c, mock_chain.net().time_service().as_ref())?;
    let (hits_before_c, reexec_before_c) = reuse_counters_for_test();
    mock_chain.apply(block_c)?;
    let c_stats = record_delta("C", hits_before_c, reexec_before_c);

    let total_vm2_txns_c = 3; // BlockMetadata + user txn + BlockEpilogue.
    assert_eq!(
        c_stats.1 + c_stats.2,
        total_vm2_txns_c,
        "block C should account for all VM2 transactions"
    );
    assert!(
        c_stats.2 >= 1,
        "block C should execute at least the user transaction, reexec_delta={}",
        c_stats.2
    );

    Ok(())
}
