// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::OpenedBlock;
use anyhow::bail;
use starcoin_accumulator::Accumulator;
use starcoin_chain_api::ExcludedTxns;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::debug;
use starcoin_types::error::BlockExecutorError;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_executor::do_execute_block_transactions;
use starcoin_vm2_state_api::ChainStateWriter;
use starcoin_vm2_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{
        SignedUserTransaction as SignedUserTransaction2, Transaction as Transaction2,
        TransactionInfo as TransactionInfo2, TransactionOutput as TransactionOutput2,
        TransactionStatus as TransactionStatus2,
    },
};

fn convert_block_meta(block_meta: starcoin_types::block_metadata::BlockMetadata) -> BlockMetadata {
    let (
        parent_hash,
        timestamp,
        author,
        _author_auth_key,
        uncles,
        number,
        chain_id,
        parent_gas_used,
    ) = block_meta.into_inner();
    let author = AccountAddress::new(author.into_bytes());
    BlockMetadata::new(
        parent_hash,
        timestamp,
        author,
        uncles,
        number,
        chain_id.id().into(),
        parent_gas_used,
    )
}

impl OpenedBlock {
    pub fn initialize2(&mut self) -> anyhow::Result<()> {
        let (_state, state) = &self.state;
        let block_metadata_txn =
            Transaction2::BlockMetadata(convert_block_meta(self.block_meta.clone()));
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = do_execute_block_transactions(
            state,
            vec![block_metadata_txn],
            None,
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
                self.push_txn_and_state2(block_meta_txn_hash, output)?;
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
        let state = &self.state.1;
        let mut txns = user_txns
            .into_iter()
            .map(Transaction2::UserTransaction)
            .collect::<Vec<_>>();
        let mut discarded_txns: Vec<MultiSignedUserTransaction> = Vec::new();
        let mut untouched_txns: Vec<MultiSignedUserTransaction> = Vec::new();

        let txn_outputs = do_execute_block_transactions(
            state,
            txns.clone(),
            Some(self.gas_limit),
            self.vm_metrics.clone(),
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;

        if txn_outputs.len() < txns.len() {
            untouched_txns = txns
                .drain(txn_outputs.len()..)
                .map(|t| t.try_into().expect("user txn"))
                .collect()
        };
        debug_assert_eq!(txns.len(), txn_outputs.len());
        for (txn, output) in txns.into_iter().zip(txn_outputs.into_iter()) {
            let txn_hash = txn.id();
            match output.status() {
                TransactionStatus2::Discard(status) => {
                    debug!("discard txn {}, vm status: {:?}", txn_hash, status);
                    discarded_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus2::Keep(status) => {
                    if !status.is_success() {
                        debug!("txn {:?} execute error: {:?}", txn_hash, status);
                    }
                    let gas_used = output.gas_used();
                    self.push_txn_and_state2(txn_hash, output)?;
                    self.gas_used += gas_used;
                    self.included_user_txns2
                        .push(txn.try_into().expect("user txn"));
                }
                TransactionStatus2::Retry => {
                    debug!("impossible retry txn {}", txn_hash);
                    discarded_txns.push(txn.try_into().expect("user txn"));
                }
            };
        }

        Ok(ExcludedTxns {
            discarded_txns,
            untouched_txns,
        })
    }

    fn push_txn_and_state2(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput2,
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
        let txn_state_root = state
            .commit()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

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
