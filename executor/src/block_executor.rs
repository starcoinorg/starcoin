// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::TransactionExecutor;
use crypto::{hash::CryptoHash, HashValue};
use logger::prelude::*;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::VMConfig;
use starcoin_state_api::ChainState;
use types::error::BlockExecutorError;
use types::error::ExecutorResult;
use types::transaction::Transaction;
use types::transaction::TransactionStatus;

pub trait BlockExecutor: std::marker::Unpin + Clone {
    /// Execute block transaction, update state to state_store, and apend accumulator , verify proof.
    fn block_execute(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        accumulator: &MerkleAccumulator,
        txns: Vec<Transaction>,
        is_preview: bool,
    ) -> ExecutorResult<()>;
}

impl BlockExecutor for Executor {
    fn block_execute(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        accumulator: &MerkleAccumulator,
        txns: Vec<Transaction>,
        is_preview: bool,
    ) -> ExecutorResult<()> {
        let mut state_root = HashValue::zero();
        let mut transaction_hash = vec![];
        for txn in txns {
            let txn_hash = txn.crypto_hash();
            let output = TransactionExecutor::execute_transaction(config, chain_state, txn)
                .map_err(|err| BlockExecutorError::BlockTransactionExecuteErr(txn_hash))?;

            match output.status() {
                TransactionStatus::Discard(status) => {
                    return Err(BlockExecutorError::BlockTransactionDiscard(
                        status.clone().into(),
                        txn_hash,
                    ))
                }
                TransactionStatus::Keep(_status) => {
                    //continue.
                }
                _ => {}
            }
            state_root = chain_state
                .commit()
                .map_err(|err| BlockExecutorError::BlockChainStateCommitErr)
                .unwrap();
            transaction_hash.push(txn_hash);
        }

        let (accumulator_root, first_leaf_idx) = accumulator
            .append(&transaction_hash)
            .map_err(|err| BlockExecutorError::BlockAccumulatorAppendErr)
            .unwrap();

        // transaction verify proof
        if !is_preview {
            transaction_hash.iter().enumerate().for_each(|(i, hash)| {
                let leaf_index = first_leaf_idx + i as u64;
                let proof = accumulator
                    .get_proof(leaf_index)
                    .map_err(|err| BlockExecutorError::BlockAccumulatorGetProofErr)
                    .unwrap()
                    .unwrap();
                proof
                    .verify(accumulator_root, *hash, leaf_index)
                    .map_err(|err| {
                        BlockExecutorError::BlockAccumulatorVerifyErr(accumulator_root, leaf_index)
                    })
                    .unwrap();
            });

            chain_state
                .flush()
                .map_err(|err| BlockExecutorError::BlockChainStateFlushErr)?;
        }

        Ok(())
    }
}
