// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::TransactionExecutor;
use crypto::{hash::PlainCryptoHash, HashValue};
// use logger::prelude::*;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::ChainState;
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
        block_gas_limit: u64,
        is_preview: bool,
    ) -> ExecutorResult<(HashValue, HashValue, Vec<TransactionInfo>)> {
        let mut state_root = HashValue::zero();
        let mut transaction_hash = vec![];
        let mut vec_transaction_info = vec![];

        // ignore for now. wait transaction output refactor.
        let _gas_left = block_gas_limit;
        let results = Executor::execute_transactions(chain_state, txns.clone())
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        for i in 0..txns.len() {
            let (txn_state_root, output) = &results[i];
            let txn_hash = txns[i].crypto_hash();
            match output.status() {
                TransactionStatus::Discard(status) => {
                    TXN_STATUS_COUNTERS.with_label_values(&["KEEP"]).inc();
                    return Err(BlockExecutorError::BlockTransactionDiscard(
                        status.clone(),
                        txn_hash,
                    ));
                }
                TransactionStatus::Keep(status) => {
                    TXN_STATUS_COUNTERS.with_label_values(&["DISCARD"]).inc();

                    // match gas_left.checked_sub(output.gas_used()) {
                    //     None => {
                    //         // now gas left is not enough to include this txn, just stop here.
                    //         break;
                    //     }
                    //     Some(left) => gas_left = left,
                    // }
                    //continue.
                    transaction_hash.push(txn_hash);
                    //TODO event root hash
                    vec_transaction_info.push(TransactionInfo::new(
                        txns[i].clone().id(),
                        *txn_state_root,
                        HashValue::zero(),
                        output.events().to_vec(),
                        output.gas_used(),
                        status.major_status,
                    ));
                }
            }
            state_root = *txn_state_root;
        }

        let (accumulator_root, first_leaf_idx) = accumulator
            .append(&transaction_hash)
            .map_err(|_err| BlockExecutorError::BlockAccumulatorAppendErr)?;

        // transaction verify proof
        if !is_preview {
            let mut i = 0;
            for hash in transaction_hash {
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
                .map_err(|_err| BlockExecutorError::BlockChainStateFlushErr)?;
        }

        Ok((accumulator_root, state_root, vec_transaction_info))
    }
}
