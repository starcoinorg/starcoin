use anyhow::format_err;
use anyhow::Result;
use executor::executor::Executor;
use executor::TransactionExecutor;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use storage::Store;
use types::account_address::AccountAddress;
use types::block::BlockHeader;
use types::block_metadata::BlockMetadata;
use types::error::BlockExecutorError;
use types::transaction::TransactionInfo;
use types::transaction::TransactionStatus;
use types::transaction::{SignedUserTransaction, Transaction};
pub struct OpenedBlock {
    previous_header: BlockHeader,
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
            state: chain_state,
            txn_accumulator,
            previous_header,
            block_meta,
            gas_limit: block_gas_limit,
            gas_left: block_gas_limit,
        };
        Ok(opened_block)
    }

    pub fn push_txns(&mut self, user_txns: Vec<SignedUserTransaction>) -> Result<NotIncludedTxns> {
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

        debug_assert_eq!(txns.len(), txn_outputs.len());
        let mut discard_txns = Vec::new();
        let mut vec_transaction_info = Vec::new();
        for (txn, output) in txns.into_iter().zip(txn_outputs.into_iter()) {
            let txn_hash = txn.id();
            let (write_set, events, gas_used, status) = output.into_inner();
            match status {
                TransactionStatus::Discard(status) => {
                    debug!("discard txn {}, vm status: {}", txn_hash, status);
                    discard_txns.push(txn);
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
                    self.txn_accumulator.append(&vec![txn_info])?;
                    self.gas_used += gas_used;
                    self.included_user_txns
                        .append(txn.try_into().expect("user txn"));

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
        todo!()
    }
}

pub struct NotIncludedTxns {
    pub discarded_txns: Vec<SignedUserTransaction>,
    pub untouched_txns: Vec<SignedUserTransaction>,
}
