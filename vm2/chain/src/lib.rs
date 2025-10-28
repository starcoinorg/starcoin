// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_executor::block_executor::{self, BlockExecutedData, VMMetrics};
use starcoin_vm2_state_api::AccountStateReader;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::block_metadata::BlockMetadata as BlockMetadata2;
use starcoin_vm2_types::error::ExecutorResult;
use starcoin_vm2_types::transaction::{SignedUserTransaction, Transaction};
use starcoin_vm2_crypto::hash::PlainCryptoHash;
use starcoin_vm2_vm_types::account_config::genesis_address;
use starcoin_vm2_vm_types::on_chain_resource::Epoch;
// reuse imports
use starcoin_exec_merge as exec_merge;
use starcoin_exec_merge::{ExecRecord, ReuseOpts};
use starcoin_vm2_types::vm_error::KeptVMStatus;

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
pub fn execute_transactions_with_reuse(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    opts: ReuseOpts,
) -> ExecutorResult<BlockExecutedData> {
    // Phase 1: 完整执行（保持对外语义完全等价）
    let executed = block_executor::block_execute(
        statedb,
        transactions.clone(),
        gas_limit,
        vm_metrics,
    )?;

    // Phase 2: 写入轻量 Witness 记录（仅用于统计，不做复用）
    let pre_fp = opts.pre_state_fingerprint;
    let store = opts.witness_store.clone();
    for (tx, info) in transactions
        .iter()
        .take(executed.txn_infos.len())
        .zip(executed.txn_infos.iter())
            {
                let rec = ExecRecord {
                    tx_hash: tx.id(),
                    pre_state_fingerprint: pre_fp,
                    read_set: None,
                    write_set: vec![],
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
