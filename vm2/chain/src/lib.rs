// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_executor::block_executor::{self, VMMetrics};
use starcoin_vm2_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_vm2_storage::Store;
use starcoin_vm2_types::transaction::{RichTransactionInfo, Transaction};

pub fn execute_txns_and_save(
    block_id: HashValue,
    block_number: u64,
    storage: &dyn Store,
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    transaction_global_index: u64,
    vm_metrics: Option<VMMetrics>,
) -> (Option<HashValue>, Vec<HashValue>) {
    // This function will execute the transactions in the block using vm2 and save the results.
    // Note: The actual implementation of VM2 execution and saving logic will depend on your VM2 setup.
    let executed_data =
        block_executor::block_execute(statedb, transactions.clone(), gas_limit, vm_metrics)
            .unwrap();

    statedb.flush().unwrap();

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
        storage.save_contract_events(*info_id, events).unwrap();
    }

    storage
        .save_transaction_infos(
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
        )
        .unwrap();

    let txn_id_vec = transactions
        .iter()
        .map(|user_txn| user_txn.id())
        .collect::<Vec<HashValue>>();
    // save transactions
    storage.save_transaction_batch(transactions).unwrap();

    // save block's transactions
    storage
        .save_block_transaction_ids(block_id, txn_id_vec)
        .unwrap();
    storage
        .save_block_txn_info_ids(block_id, txn_info_ids)
        .unwrap();
    storage.save_table_infos(txn_table_infos).unwrap();

    let included_txn_info_hashes: Vec<_> =
        txn_infos.iter().map(|info| info.id()).collect::<Vec<_>>();
    (Some(executed_data.state_root), included_txn_info_hashes)
}
