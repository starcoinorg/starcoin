// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_state_api::ChainState;
use starcoin_types::error::BlockExecutorError;
use starcoin_types::error::ExecutorResult;
use starcoin_types::transaction::TransactionStatus;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use vm_runtime::metrics::TXN_STATUS_COUNTERS;

pub fn block_execute(
    chain_state: &dyn ChainState,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
) -> ExecutorResult<(HashValue, Vec<TransactionInfo>)> {
    let mut vec_transaction_info = vec![];
    let txn_outputs =
        crate::execute_block_transactions(chain_state.as_super(), txns.clone(), block_gas_limit)
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;

    for (txn, output) in txns
        .iter()
        .take(txn_outputs.len())
        .zip(txn_outputs.into_iter())
    {
        let txn_hash = txn.id();
        let (write_set, events, gas_used, _, status) = output.into_inner();
        match status {
            TransactionStatus::Discard(status) => {
                TXN_STATUS_COUNTERS.with_label_values(&["DISCARD"]).inc();
                return Err(BlockExecutorError::BlockTransactionDiscard(
                    status, txn_hash,
                ));
            }
            TransactionStatus::Keep(status) => {
                TXN_STATUS_COUNTERS.with_label_values(&["KEEP"]).inc();
                chain_state
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;

                let txn_state_root = chain_state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?;

                vec_transaction_info.push(TransactionInfo::new(
                    txn_hash,
                    txn_state_root,
                    events,
                    gas_used,
                    status.major_status,
                ));
            }
        };
    }
    Ok((chain_state.state_root(), vec_transaction_info))
}
