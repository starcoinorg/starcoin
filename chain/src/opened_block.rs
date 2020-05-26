use anyhow::{bail, format_err, Result};
use crypto::hash::{HashValue, PlainCryptoHash};
use executor::{executor::Executor, TransactionExecutor};
use logger::prelude::*;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use std::{
    convert::TryInto,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use storage::Store;
use traits::ExcludedTxns;
use types::{
    account_address::AccountAddress,
    block::{BlockHeader, BlockInfo, BlockTemplate},
    block_metadata::BlockMetadata,
    error::BlockExecutorError,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
};

pub struct OpenedBlock {
    previous_header: BlockHeader,
    previous_block_info: BlockInfo,
    block_meta: BlockMetadata,
    gas_limit: u64,

    state: ChainStateDB,
    txn_accumulator: MerkleAccumulator,

    gas_used: u64,
    included_user_txns: Vec<SignedUserTransaction>,
}

impl OpenedBlock {
    pub fn new<S>(
        storage: Arc<S>,
        previous_header: BlockHeader,
        block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
    ) -> Result<Self>
    where
        S: Store + 'static,
    {
        let previous_block_id = previous_header.id();
        let block_info = storage
            .get_block_info(previous_block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", previous_block_id))?;
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new(
            *txn_accumulator_info.get_accumulator_root(),
            txn_accumulator_info.get_frozen_subtree_roots().clone(),
            txn_accumulator_info.get_num_leaves(),
            txn_accumulator_info.get_num_nodes(),
            storage.clone(),
        )?;
        let block_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let chain_state = ChainStateDB::new(storage, Some(previous_header.state_root()));
        let block_meta =
            BlockMetadata::new(previous_block_id, block_timestamp, author, auth_key_prefix);
        let opened_block = Self {
            previous_header,
            previous_block_info: block_info,
            block_meta,
            gas_limit: block_gas_limit,

            state: chain_state,
            txn_accumulator,
            gas_used: 0,
            included_user_txns: vec![],
        };
        Ok(opened_block)
    }

    /// Try to add `user_txns` into this block.
    /// Return any txns  not included, either txn is discarded, or block gas limit is reached.
    /// If error occurs during the processing, the `open_block` should be dropped,
    /// as the internal state may be corrupted.
    /// TODO: make the function can be called again even last call returns error.  
    pub fn push_txns(&mut self, user_txns: Vec<SignedUserTransaction>) -> Result<ExcludedTxns> {
        let mut txns: Vec<_> = user_txns
            .iter()
            .cloned()
            .map(Transaction::UserTransaction)
            .collect();

        let txn_outputs = {
            let gas_left = self
                .gas_limit
                .checked_sub(self.gas_used)
                .expect("block gas_used exceed block gas_limit");
            Executor::execute_block_transactions(&self.state, txns.clone(), gas_left)?
        };

        let untouched_user_txns: Vec<SignedUserTransaction> = if txn_outputs.len() >= txns.len() {
            vec![]
        } else {
            txns.drain(txn_outputs.len()..)
                .map(|t| t.try_into().expect("user txn"))
                .collect()
        };

        let mut discard_txns: Vec<SignedUserTransaction> = Vec::new();
        debug_assert_eq!(txns.len(), txn_outputs.len());
        for (txn, output) in txns.into_iter().zip(txn_outputs.into_iter()) {
            let txn_hash = txn.id();
            let (write_set, events, gas_used, status) = output.into_inner();
            match status {
                TransactionStatus::Discard(status) => {
                    debug!("discard txn {}, vm status: {}", txn_hash, status);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus::Keep(status) => {
                    // push txn, and update state
                    self.state
                        .apply_write_set(write_set)
                        .map_err(BlockExecutorError::BlockChainStateErr)?;
                    let txn_state_root = self
                        .state
                        .commit()
                        .map_err(BlockExecutorError::BlockChainStateErr)?;
                    let txn_info = TransactionInfo::new(
                        txn_hash,
                        txn_state_root,
                        //TODO add event root hash.
                        HashValue::zero(),
                        events,
                        gas_used,
                        status.major_status,
                    );
                    // NOTICE: txn_accumulator's leave is txn_info, not txn.
                    self.txn_accumulator.append(&[txn_info.crypto_hash()])?;
                    self.gas_used += gas_used;
                    self.included_user_txns
                        .push(txn.try_into().expect("user txn"));
                }
            };
        }
        Ok(ExcludedTxns {
            discarded_txns: discard_txns,
            untouched_txns: untouched_user_txns,
        })
    }

    /// Construct a block template for mining.
    pub fn finalize(self) -> Result<BlockTemplate> {
        let block_metadata_txn = Transaction::BlockMetadata(self.block_meta.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = Executor::execute_transactions(&self.state, vec![block_metadata_txn])
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");
        let (write_set, events, gas_used, status) = output.into_inner();
        let (state_root, accumulator_root) = match status {
            TransactionStatus::Discard(status) => {
                bail!("block_metadata txn is discarded, vm status: {}", status);
            }
            TransactionStatus::Keep(status) => {
                debug_assert_eq!(gas_used, 0, "execute block meta should not use any gas");

                self.state
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                let txn_state_root = self
                    .state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                let txn_info = TransactionInfo::new(
                    block_meta_txn_hash,
                    txn_state_root,
                    //TODO add event root hash.
                    HashValue::zero(),
                    events,
                    gas_used,
                    status.major_status,
                );
                let (accumulator_root, _) =
                    self.txn_accumulator.append(&[txn_info.crypto_hash()])?;
                (txn_state_root, accumulator_root)
            }
        };
        let (parent_id, timestamp, author, auth_key_prefix) = self.block_meta.into_inner();

        let block_template = BlockTemplate::new(
            parent_id,
            self.previous_block_info
                .block_accumulator_info
                .accumulator_root,
            timestamp,
            self.previous_header.number() + 1,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            self.gas_used,
            self.gas_limit,
            self.included_user_txns.into(),
        );
        Ok(block_template)
    }
}
