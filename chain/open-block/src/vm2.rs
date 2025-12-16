// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::OpenedBlock;
use anyhow::{bail, format_err};
use starcoin_accumulator::Accumulator;
use starcoin_chain_api::ExcludedTxns;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{debug, info};
use starcoin_types::error::BlockExecutorError;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_executor::do_execute_block_transactions;
use starcoin_vm2_state_api::ChainStateWriter;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::transaction::{
    SignedUserTransaction as SignedUserTransaction2, Transaction as Transaction2,
    TransactionInfo as TransactionInfo2, TransactionOutput as TransactionOutput2,
    TransactionStatus as TransactionStatus2,
};
use std::collections::BTreeSet;

impl OpenedBlock {
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        let (_state, state) = &self.state;
        // Directly use VM2 BlockMetadata
        let block_metadata_txn = Transaction2::BlockMetadata(self.block_meta.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = do_execute_block_transactions(
            state,
            vec![block_metadata_txn],
            Some(self.gas_limit()),
            self.vm_metrics.clone(),
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");

        match output.status() {
            TransactionStatus2::Discard(status) => {
                bail!(
                    "block_metadata txn {:?} is discarded, vm status: {:?}",
                    self.block_meta,
                    status
                );
            }
            TransactionStatus2::Keep(_) => {
                // Cache BlockMetadata output for reuse during block execution
                self.cached_vm2_outputs.push(output.clone());
                self.push_txn_and_state2(block_meta_txn_hash, output, true)?;
            }
            TransactionStatus2::Retry => {
                bail!(
                    "block_metadata txn {:?} is retry impossible",
                    self.block_meta
                );
            }
        };
        Ok(())
    }
    pub fn push_txns2(
        &mut self,
        user_txns: Vec<SignedUserTransaction2>,
    ) -> anyhow::Result<ExcludedTxns> {
        let start_time = std::time::Instant::now();
        let input_count = user_txns.len();
        info!(
            "[jacktest] push_txns2 start: input_count={}, gas_used={}, gas_limit={}",
            input_count, self.gas_used, self.gas_limit
        );
        let state = &self.state.1;
        let mut txns = user_txns
            .into_iter()
            .map(Transaction2::UserTransaction)
            .collect::<Vec<_>>();
        let mut discarded_txns: Vec<MultiSignedUserTransaction> = Vec::new();
        let mut untouched_txns: Vec<MultiSignedUserTransaction> = Vec::new();

        let exec_start = std::time::Instant::now();
        let txn_outputs = {
            let gas_left = self.gas_limit.checked_sub(self.gas_used).ok_or_else(|| {
                format_err!(
                    "block gas_used {} exceed block gas_limit:{}",
                    self.gas_used,
                    self.gas_limit
                )
            })?;
            do_execute_block_transactions(
                state,
                txns.clone(),
                // Some(gas_left),
                Some(1_000_000_000),
                self.vm_metrics.clone(),
            )
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?
        };
        info!(
            "[jacktest] push_txns2 execution done: outputs={}, exec_elapsed_ms={}",
            txn_outputs.len(),
            exec_start.elapsed().as_millis()
        );

        let gas_exceeded_count = txns.len().saturating_sub(txn_outputs.len());
        if txn_outputs.len() < txns.len() {
            untouched_txns = txns
                .drain(txn_outputs.len()..)
                .map(|t| t.try_into().expect("user txn"))
                .collect()
        };
        debug_assert_eq!(txns.len(), txn_outputs.len());
        
        let mut keep_count = 0;
        let mut discard_count = 0;
        let mut retry_count = 0;
        
        for (index, (txn, output)) in txns.into_iter().zip(txn_outputs.into_iter()).enumerate() {
            let txn_hash = txn.id();
            match output.status() {
                TransactionStatus2::Discard(status) => {
                    discard_count += 1;
                    if index < 5 {
                        info!(
                            "[jacktest] push_txns2 tx #{}: hash=0x{}, status=discard, reason={:?}",
                            index, &txn_hash.to_string()[..6], status
                        );
                    }
                    debug!("discard txn {}, vm status: {:?}", txn_hash, status);
                    discarded_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus2::Keep(status) => {
                    keep_count += 1;
                    if !status.is_success() {
                        debug!("txn {:?} execute error: {:?}", txn_hash, status);
                    }
                    let gas_used = output.gas_used();
                    if index < 5 || index % 500 == 0 {
                        info!(
                            "[jacktest] push_txns2 tx #{}: hash=0x{}, status=keep, gas_used={}, total_gas={}",
                            index, &txn_hash.to_string()[..6], gas_used, self.gas_used + gas_used
                        );
                    }
                    self.cached_vm2_outputs.push(output.clone());
                    self.push_txn_and_state2(txn_hash, output, false)?;
                    self.gas_used += gas_used;
                    self.included_user_txns2
                        .push(txn.try_into().expect("user txn"));
                }
                TransactionStatus2::Retry => {
                    retry_count += 1;
                    debug!("impossible retry txn {}", txn_hash);
                    discarded_txns.push(txn.try_into().expect("user txn"));
                }
            };
        }

        info!(
            "[jacktest] push_txns2 done: input={}, keep={}, discard={}, retry={}, gas_exceeded={}, final_gas={}, elapsed_ms={}",
            input_count,
            keep_count,
            discard_count,
            retry_count,
            gas_exceeded_count,
            self.gas_used,
            start_time.elapsed().as_millis()
        );

        Ok(ExcludedTxns {
            discarded_txns,
            untouched_txns,
        })
    }

    pub fn finalize_block_epilogue(&mut self) -> anyhow::Result<()> {
        let (_state, state) = &self.state;
        // Directly use VM2 BlockEpilogue
        let senders: BTreeSet<AccountAddress> = self
            .included_user_txns2
            .iter()
            .map(|txn| txn.sender())
            .collect();
        let block_epilogue_txn = Transaction2::BlockEpilogue(self.block_meta.clone(), senders);
        let block_epilogue_txn_hash = block_epilogue_txn.id();
        let mut results = do_execute_block_transactions(
            state,
            vec![block_epilogue_txn],
            Some(self.gas_limit()),
            self.vm_metrics.clone(),
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");

        match output.status() {
            TransactionStatus2::Discard(status) => {
                bail!(
                    "block_epilogue txn {:?} is discarded, vm status: {:?}",
                    self.block_meta,
                    status
                );
            }
            TransactionStatus2::Keep(_) => {
                // Cache BlockEpilogue output for reuse during block execution
                self.cached_vm2_outputs.push(output.clone());
                self.push_txn_and_state2(block_epilogue_txn_hash, output, true)?;
            }
            TransactionStatus2::Retry => {
                bail!(
                    "block_epilogue txn {:?} is retry impossible",
                    self.block_meta
                );
            }
        };
        Ok(())
    }

    fn push_txn_and_state2(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput2,
        state_root_calc: bool,
    ) -> anyhow::Result<()> {
        let state = &mut self.state.1;
        let (write_set, events, gas_used, status, _) = output.into_inner();
        debug_assert!(matches!(status, TransactionStatus2::Keep(_)));
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
        state
            .apply_write_set(write_set)
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        let txn_state_root = if state_root_calc {
            Some(
                state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?,
            )
        } else {
            None
        };

        let txn_info = TransactionInfo2::new(
            txn_hash,
            txn_state_root,
            events.as_slice(),
            gas_used,
            status,
        );
        self.txn_accumulator.append(&[txn_info.id()])?;
        Ok(())
    }
}
