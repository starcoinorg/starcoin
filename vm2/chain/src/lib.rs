// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_crypto::hash::PlainCryptoHash;
use starcoin_vm2_executor::block_executor::{self, BlockExecutedData, VMMetrics};
use starcoin_vm2_state_api::AccountStateReader;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::block_metadata::BlockMetadata as BlockMetadata2;
use starcoin_vm2_types::error::ExecutorResult;
use starcoin_vm2_types::transaction::{SignedUserTransaction, Transaction};
use starcoin_vm2_vm_types::account_config::genesis_address;
use starcoin_vm2_vm_types::on_chain_resource::Epoch;
// reuse imports
use starcoin_exec_merge as exec_merge;
use starcoin_exec_merge::{ExecRecord, ReuseOpts};
use starcoin_vm2_types::vm_error::KeptVMStatus;
use starcoin_vm_runtime::reuse_recorder;

pub fn execute_transactions(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> ExecutorResult<BlockExecutedData> {
    // This function will execute the transactions in the block using vm2
    // Note: The actual implementation of VM2 execution and saving logic will depend on your VM2 setup.
    let executed_data =
        block_executor::block_execute(statedb, transactions, gas_limit, vm_metrics)?;

    Ok(executed_data)
}

/// Execute with reuse fast-path (skeleton). When `opts.enabled` is false, falls back to execute_transactions.
fn to_read_entries(reads: Vec<reuse_recorder::ReadDescriptor>) -> Vec<exec_merge::ReadEntry> {
    reads
        .into_iter()
        .map(|desc| exec_merge::ReadEntry {
            key: desc.key,
            from_storage: desc.from_storage,
            existed: desc.existed,
            value_hash: desc.value_hash,
        })
        .collect()
}

pub fn execute_transactions_with_reuse(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    opts: ReuseOpts,
) -> ExecutorResult<BlockExecutedData> {
    let recording = opts.enabled;
    let (executed, recorded_reads) = if recording {
        reuse_recorder::start();
        let exec_res =
            block_executor::block_execute(statedb, transactions.clone(), gas_limit, vm_metrics);
        match exec_res {
            Ok(data) => (data, reuse_recorder::finish()),
            Err(err) => {
                reuse_recorder::finish();
                return Err(err);
            }
        }
    } else {
        (
            block_executor::block_execute(statedb, transactions.clone(), gas_limit, vm_metrics)?,
            Vec::<Vec<reuse_recorder::ReadDescriptor>>::new(),
        )
    };

    // Phase 2: 写入 Witness 记录
    let pre_fp = opts.pre_state_fingerprint;
    let store = opts.witness_store.clone();
    for (idx, (tx, info)) in transactions
        .iter()
        .take(executed.txn_infos.len())
        .zip(executed.txn_infos.iter())
        .enumerate()
    {
        let write_set_entries = executed
            .write_sets
            .get(idx)
            .map(|ws| ws.clone().into_iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let read_set_entries = recorded_reads.get(idx).cloned().map(to_read_entries);

        let rec = ExecRecord {
            tx_hash: tx.id(),
            pre_state_fingerprint: pre_fp,
            read_set: read_set_entries,
            write_set: write_set_entries,
            event_root: info.event_root_hash(),
            gas: info.gas_used(),
            status_ok: matches!(info.status(), KeptVMStatus::Executed),
            meta_fingerprint: None,
        };
        store.put(rec);
    }

    Ok(executed)
}

pub fn build_block_transactions(
    signed_txns: &[SignedUserTransaction],
    block_meta: Option<BlockMetadata>,
) -> Vec<Transaction> {
    let mut txns = block_meta
        .as_ref()
        .map(|m| vec![Transaction::BlockMetadata(m.clone())])
        .unwrap_or_default();
    txns.extend(
        signed_txns
            .iter()
            .map(|t| Transaction::UserTransaction(t.clone())),
    );

    // contains user transaction
    if txns.len() > 1 {
        let senders = signed_txns.iter().map(|t| t.sender()).collect();
        txns.extend(
            block_meta
                .map(|m| vec![Transaction::BlockEpilogue(m, senders)])
                .unwrap_or_default(),
        );
    }
    txns
}

/// Helper to compute pre-state fingerprint from parent root and BlockMetadata.
pub fn create_pre_state_fingerprint(
    parent_state_root2: starcoin_vm2_crypto::HashValue,
    metadata: &BlockMetadata2,
    epoch_version: u64,
) -> starcoin_vm2_crypto::HashValue {
    let meta_hash = metadata.crypto_hash();
    exec_merge::create_pre_state_fingerprint(parent_state_root2, meta_hash, epoch_version)
}

pub fn get_epoch_from_statedb(statedb: &ChainStateDB) -> anyhow::Result<Epoch> {
    let account_reader = AccountStateReader::new(statedb);
    account_reader.get_resource::<Epoch>(genesis_address())
}
