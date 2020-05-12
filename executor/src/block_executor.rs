// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::TransactionExecutor;
use crypto::{hash::PlainCryptoHash, HashValue};
// use logger::prelude::*;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::ChainState;
use types::error::BlockExecutorError;
use types::error::ExecutorResult;
use types::transaction::TransactionStatus;
use types::transaction::{Transaction, TransactionInfo};

#[derive(Clone)]
pub struct BlockExecutor {}

impl BlockExecutor {
    /// Execute block transaction, update state to state_store, and apend accumulator , verify proof.
    pub fn block_execute(
        chain_state: &dyn ChainState,
        accumulator: &MerkleAccumulator,
        txns: Vec<Transaction>,
        is_preview: bool,
    ) -> ExecutorResult<(HashValue, HashValue, Vec<TransactionInfo>)> {
        let mut state_root = HashValue::zero();
        let mut transaction_hash = vec![];
        let mut vec_transaction_info = vec![];
        for txn in txns {
            let txn_hash = txn.crypto_hash();
            let output = Executor::execute_transaction(chain_state, txn.clone())
                .map_err(|_err| BlockExecutorError::BlockTransactionExecuteErr(txn_hash))?;

            match output.status() {
                TransactionStatus::Discard(status) => {
                    return Err(BlockExecutorError::BlockTransactionDiscard(
                        status.clone().into(),
                        txn_hash,
                    ))
                }
                TransactionStatus::Keep(status) => {
                    //continue.
                    transaction_hash.push(txn_hash);
                    //TODO event root hash
                    vec_transaction_info.push(TransactionInfo::new(
                        txn.clone().id(),
                        state_root.clone(),
                        HashValue::zero(),
                        output.gas_used(),
                        status.major_status,
                    ));
                }
            }
            state_root = chain_state
                .commit()
                .map_err(|_err| BlockExecutorError::BlockChainStateCommitErr)?;
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
