// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_executor::block_executor::{self, BlockExecutedData, VMMetrics};
use starcoin_vm2_state_api::AccountStateReader;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_storage::Store;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::error::ExecutorResult;
use starcoin_vm2_types::transaction::{SignedUserTransaction, Transaction};
use starcoin_vm2_vm_types::account_config::genesis_address;
use starcoin_vm2_vm_types::on_chain_resource::Epoch;

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

// todo: remove me.
pub fn save_executed_transactions(
    storage: &dyn Store,
    executed_data: BlockExecutedData,
) -> anyhow::Result<()> {
    let txn_table_infos = executed_data
        .txn_table_infos
        .into_iter()
        .collect::<Vec<_>>();

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

pub fn get_epoch_from_statedb(statedb: &ChainStateDB) -> anyhow::Result<Epoch> {
    let account_reader = AccountStateReader::new(statedb);
    account_reader.get_resource::<Epoch>(genesis_address())
}
