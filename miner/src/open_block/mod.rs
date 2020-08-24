use actix::{Actor, Addr, Context};
use anyhow::{bail, format_err, Result};
use crypto::hash::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_vm_types::on_chain_config::EpochInfo;
use statedb::ChainStateDB;
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use storage::Store;
use traits::ExcludedTxns;
use types::{
    account_address::AccountAddress,
    block::{BlockBody, BlockHeader, BlockInfo, BlockTemplate},
    block_metadata::BlockMetadata,
    error::BlockExecutorError,
    transaction::{
        SignedUserTransaction, Transaction, TransactionInfo, TransactionOutput, TransactionStatus,
    },
    vm_error::KeptVMStatus,
};

pub struct OpenBlockActor {
    inner: Inner,
}

impl OpenBlockActor {
    pub fn launch(head_block_info: BlockInfo) -> Result<Addr<Self>> {
        let actor = OpenBlockActor::create(move |ctx| OpenBlockActor {
            inner: Inner::new(head_block_info),
        });
        Ok(actor)
    }
}

impl Actor for OpenBlockActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("OpenBlockActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("OpenBlockActor stopped");
    }
}

struct Inner {
    head_block_info: BlockInfo,
    parent_uncle: HashMap<HashValue, Vec<BlockHeader>>,
    uncles: HashMap<HashValue, BlockHeader>,
    epoch_data: Option<EpochInfo>,
}

impl Inner {
    fn new(head_block_info: BlockInfo) -> Self {
        Inner {
            head_block_info,
            parent_uncle: HashMap::new(),
            uncles: HashMap::new(),
            epoch_data: None,
        }
    }
}

pub struct OpenedBlock {
    previous_block_info: BlockInfo,
    block_meta: BlockMetadata,
    gas_limit: u64,

    state: ChainStateDB,
    txn_accumulator: MerkleAccumulator,

    gas_used: u64,
    included_user_txns: Vec<SignedUserTransaction>,
    uncles: Vec<BlockHeader>,
}

impl OpenedBlock {
    pub fn new(
        storage: Arc<dyn Store>,
        previous_header: BlockHeader,
        block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        block_timestamp: u64,
        uncles: Vec<BlockHeader>,
    ) -> Result<Self> {
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
            AccumulatorStoreType::Transaction,
            storage.clone().into_super_arc(),
        )?;
        // let block_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let chain_state =
            ChainStateDB::new(storage.into_super_arc(), Some(previous_header.state_root()));
        let block_meta = BlockMetadata::new(
            previous_block_id,
            block_timestamp,
            author,
            auth_key_prefix,
            uncles.len() as u64,
            previous_header.number + 1,
        );
        let mut opened_block = Self {
            previous_block_info: block_info,
            block_meta,
            gas_limit: block_gas_limit,

            state: chain_state,
            txn_accumulator,
            gas_used: 0,
            included_user_txns: vec![],
            uncles,
        };
        opened_block.initialize()?;
        Ok(opened_block)
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    // TODO: should use check_sub or not
    pub fn gas_left(&self) -> u64 {
        debug_assert!(self.gas_limit >= self.gas_used);
        self.gas_limit - self.gas_used
    }

    pub fn included_user_txns(&self) -> &[SignedUserTransaction] {
        &self.included_user_txns
    }
    pub fn state_root(&self) -> HashValue {
        self.state.state_root()
    }
    pub fn accumulator_root(&self) -> HashValue {
        self.txn_accumulator.root_hash()
    }

    pub fn block_meta(&self) -> &BlockMetadata {
        &self.block_meta
    }
    pub fn block_number(&self) -> u64 {
        self.block_meta.number()
    }

    pub fn state_reader(&self) -> &impl ChainStateReader {
        &self.state
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
            executor::execute_block_transactions(&self.state, txns.clone(), gas_left)?
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
            match output.status() {
                TransactionStatus::Discard(status) => {
                    debug!("discard txn {}, vm status: {:?}", txn_hash, status);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus::Keep(status) => {
                    if status != &KeptVMStatus::Executed {
                        debug!("txn {:?} execute error: {:?}", txn_hash, status);
                    }
                    let gas_used = output.gas_used();
                    self.push_txn_and_state(txn_hash, output)?;
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

    /// Run blockmeta first
    fn initialize(&mut self) -> Result<()> {
        let block_metadata_txn = Transaction::BlockMetadata(self.block_meta.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = executor::execute_transactions(&self.state, vec![block_metadata_txn])
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");

        match output.status() {
            TransactionStatus::Discard(status) => {
                bail!("block_metadata txn is discarded, vm status: {:?}", status);
            }
            TransactionStatus::Keep(_) => {
                let _ = self.push_txn_and_state(block_meta_txn_hash, output)?;
            }
        };
        Ok(())
    }

    fn push_txn_and_state(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput,
    ) -> Result<(HashValue, HashValue)> {
        let (write_set, events, gas_used, _, status) = output.into_inner();
        debug_assert!(matches!(status, TransactionStatus::Keep(_)));
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
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
            events.as_slice(),
            gas_used,
            status,
        );
        let (accumulator_root, _) = self.txn_accumulator.append(&[txn_info.id()])?;
        Ok((txn_state_root, accumulator_root))
    }

    /// Construct a block template for mining.
    pub fn finalize(self) -> Result<BlockTemplate> {
        let accumulator_root = self.txn_accumulator.root_hash();
        let state_root = self.state.state_root();
        let (parent_id, timestamp, author, auth_key_prefix, _uncles, number) =
            self.block_meta.into_inner();

        let (uncle_hash, uncles) = if !self.uncles.is_empty() {
            (
                Some(HashValue::sha3_256_of(&self.uncles.encode()?)),
                Some(self.uncles),
            )
        } else {
            (None, None)
        };
        let body = BlockBody::new(self.included_user_txns, uncles);
        let block_template = BlockTemplate::new(
            parent_id,
            self.previous_block_info
                .block_accumulator_info
                .accumulator_root,
            timestamp,
            number,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            self.gas_used,
            self.gas_limit,
            uncle_hash,
            body,
        );
        Ok(block_template)
    }
}
