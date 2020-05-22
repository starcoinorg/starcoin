// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::TransactionExecutor;
use crypto::HashValue;
// use logger::prelude::*;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::ChainState;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::error::BlockExecutorError;
use starcoin_types::error::ExecutorResult;
use starcoin_types::transaction::TransactionStatus;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use vm_runtime::counters::TXN_STATUS_COUNTERS;

#[derive(Clone)]
pub struct BlockExecutor {}

impl BlockExecutor {
    /// Execute block transaction, update state to state_store, and apend accumulator , verify proof.
    pub fn block_execute(
        chain_state: &dyn ChainState,
        accumulator: &MerkleAccumulator,
        txns: Vec<Transaction>,
        block_metadata: BlockMetadata,
        block_gas_limit: u64,
        is_preview: bool,
    ) -> ExecutorResult<(HashValue, HashValue, Vec<TransactionInfo>)> {
        let mut vec_transaction_info = vec![];
        let mut included_txn_hashes = vec![];
        // ignore for now. wait transaction output refactor.
        let mut gas_left = block_gas_limit;
        for txn in txns {
            let txn_hash = txn.id();
            // execute txn one by one, and check block gas limit before commit the transaction output.
            let mut results = Executor::execute_transactions(chain_state.as_super(), vec![txn])
                .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
            let output = results.pop().expect("execute txn has output");
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

                    // check block gas
                    match gas_left.checked_sub(gas_used) {
                        None => {
                            // now gas left is not enough to include this txn, just stop here.
                            break;
                        }
                        Some(left) => gas_left = left,
                    }

                    chain_state
                        .apply_write_set(write_set)
                        .map_err(BlockExecutorError::BlockChainStateErr)?;
                    let txn_state_root = chain_state
                        .commit()
                        .map_err(BlockExecutorError::BlockChainStateErr)?;
                    //continue.
                    included_txn_hashes.push(txn_hash);
                    vec_transaction_info.push(TransactionInfo::new(
                        txn_hash,
                        txn_state_root,
                        //TODO add event root hash.
                        HashValue::zero(),
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
                //continue.
                included_txn_hashes.push(block_meta_txn_hash);
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

        let (accumulator_root, first_leaf_idx) = accumulator
            .append(&included_txn_hashes)
            .map_err(|_err| BlockExecutorError::BlockAccumulatorAppendErr)?;

        // TODO: move the proof somewhere else.
        // transaction verify proof
        if !is_preview {
            let mut i = 0;
            for hash in included_txn_hashes {
                let leaf_index = first_leaf_idx + i as u64;
                if let Some(proof) = accumulator
                    .get_proof(leaf_index)
                    .map_err(|_err| BlockExecutorError::BlockAccumulatorGetProofErr)?
                {
                    proof
                        .verify(accumulator_root, hash, leaf_index)
                        .map_err(|_err| {
                            BlockExecutorError::BlockAccumulatorVerifyErr(
                                accumulator_root,
                                leaf_index,
                            )
                        })?;
                }
                i += 1;
            }

            accumulator
                .flush()
                .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;
            chain_state
                .flush()
                .map_err(BlockExecutorError::BlockChainStateErr)?;
        }

        Ok((accumulator_root, state_root, vec_transaction_info))
    }
}
