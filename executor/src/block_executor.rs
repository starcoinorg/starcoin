// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::TransactionExecutor;
use crypto::HashValue;
// use logger::prelude::*;
use crypto::hash::PlainCryptoHash;
use starcoin_state_api::ChainState;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::contract_event::ContractEventHasher;
use starcoin_types::error::BlockExecutorError;
use starcoin_types::error::ExecutorResult;
use starcoin_types::proof::InMemoryAccumulator;
use starcoin_types::transaction::TransactionStatus;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use vm_runtime::counters::TXN_STATUS_COUNTERS;

#[derive(Clone)]
pub struct BlockExecutor {}

impl BlockExecutor {
    /// Execute block transaction, only update state in cache.
    /// Caller should decide whether flush or not.
    pub fn block_execute(
        chain_state: &dyn ChainState,
        txns: Vec<Transaction>,
        block_metadata: BlockMetadata,
        block_gas_limit: u64,
    ) -> ExecutorResult<(HashValue, Vec<TransactionInfo>)> {
        let mut vec_transaction_info = vec![];
        let txn_outputs = Executor::execute_block_transactions(
            chain_state.as_super(),
            txns.clone(),
            block_gas_limit,
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;

        for (txn, output) in txns
            .iter()
            .take(txn_outputs.len())
            .zip(txn_outputs.into_iter())
        {
            let txn_hash = txn.id();
            let (write_set, events, gas_used, status) = output.into_inner();
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
                    let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();
                    let events_accumulator_hash =
                        InMemoryAccumulator::<ContractEventHasher>::from_leaves(
                            event_hashes.as_slice(),
                        )
                        .root_hash();

                    vec_transaction_info.push(TransactionInfo::new(
                        txn_hash,
                        txn_state_root,
                        events_accumulator_hash,
                        events,
                        gas_used,
                        status.major_status,
                    ));
                }
            };
        }

        // now we execute block meta txn
        let block_metadata_txn = Transaction::BlockMetadata(block_metadata);
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results =
            Executor::execute_transactions(chain_state.as_super(), vec![block_metadata_txn])
                .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");
        let (write_set, events, gas_used, status) = output.into_inner();
        let state_root = match status {
            TransactionStatus::Discard(status) => {
                TXN_STATUS_COUNTERS.with_label_values(&["DISCARD"]).inc();
                return Err(BlockExecutorError::BlockTransactionDiscard(
                    status,
                    block_meta_txn_hash,
                ));
            }
            TransactionStatus::Keep(status) => {
                TXN_STATUS_COUNTERS.with_label_values(&["KEEP"]).inc();

                assert_eq!(gas_used, 0, "execute block meta should not use any gas");

                chain_state
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                let txn_state_root = chain_state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                vec_transaction_info.push(TransactionInfo::new(
                    block_meta_txn_hash,
                    txn_state_root,
                    //TODO add event root hash.
                    HashValue::zero(),
                    events,
                    gas_used,
                    status.major_status,
                ));
                txn_state_root
            }
        };
        Ok((state_root, vec_transaction_info))
    }
}
