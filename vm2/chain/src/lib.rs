// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_executor::block_executor::{self, BlockExecutedData, VMMetrics};
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_storage::Store;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::error::ExecutorResult;
use starcoin_vm2_types::transaction::{RichTransactionInfo, SignedUserTransaction, Transaction};

pub fn execute_transactions(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> ExecutorResult<(BlockExecutedData, Vec<HashValue>)> {
    // This function will execute the transactions in the block using vm2
    // Note: The actual implementation of VM2 execution and saving logic will depend on your VM2 setup.
    let executed_data =
        block_executor::block_execute(statedb, transactions, gas_limit, vm_metrics)?;

    let included_txn_info_hashes: Vec<_> = executed_data
        .txn_infos
        .iter()
        .map(|info| info.id())
        .collect::<Vec<_>>();

    Ok((executed_data, included_txn_info_hashes))
}

pub fn save_executed_transactions(
    block_id: HashValue,
    block_number: u64,
    storage: &dyn Store,
    transactions: Vec<Transaction>,
    executed_data: BlockExecutedData,
    transaction_global_index: u64,
) -> anyhow::Result<()> {
    // Save the state root and transaction info to the database.
    let txn_infos = executed_data.txn_infos;
    let txn_events = executed_data.txn_events;
    let txn_table_infos = executed_data
        .txn_table_infos
        .into_iter()
        .collect::<Vec<_>>();

    debug_assert!(
        txn_events.len() == txn_infos.len(),
        "events' length should be equal to txn infos' length"
    );
    let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
    for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
        storage.save_contract_events(*info_id, events)?;
    }

    storage.save_transaction_infos(
        txn_infos
            .iter()
            .enumerate()
            .map(|(transaction_index, info)| {
                RichTransactionInfo::new(
                    block_id,
                    block_number,
                    info.clone(),
                    transaction_index as u32,
                    transaction_global_index
                        .checked_add(transaction_index as u64)
                        .expect("transaction_global_index overflow."),
                )
            })
            .collect(),
    )?;

    let txn_id_vec = transactions
        .iter()
        .map(|user_txn| user_txn.id())
        .collect::<Vec<HashValue>>();
    // save transactions
    storage.save_transaction_batch(transactions)?;

    // save block's transactions
    storage.save_block_transaction_ids(block_id, txn_id_vec)?;
    storage.save_block_txn_info_ids(block_id, txn_info_ids)?;
    storage.save_table_infos(txn_table_infos)?;

    Ok(())
}

pub fn build_block_transactions(
    signed_txns: &[SignedUserTransaction],
    block_meta: Option<BlockMetadata>,
) -> Vec<Transaction> {
    let mut txns = block_meta
        .map(|m| vec![Transaction::BlockMetadata(m)])
        .unwrap_or_default();
    txns.extend(
        signed_txns
            .iter()
            .map(|t| Transaction::UserTransaction(t.clone())),
    );
    txns
}
