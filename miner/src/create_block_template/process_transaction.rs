use std::sync::Arc;

use anyhow::{bail, format_err};
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_executor::{execute_block_transactions, execute_transactions, VMMetrics};
use starcoin_logger::prelude::{debug, info};
use starcoin_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_storage::Store;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::BlockHeader,
    block_metadata::BlockMetadata,
    error::BlockExecutorError,
    genesis_config::ConsensusStrategy,
    transaction::{
        SignedUserTransaction, Transaction, TransactionInfo, TransactionOutput, TransactionStatus,
    },
    vm_error::KeptVMStatus,
    U256,
};

use super::block_builder_service::TemplateTxProvider;

#[derive(Clone)]
pub struct ProcessHeaderTemplate {
    pub header: BlockHeader,
    pub uncles: Vec<BlockHeader>,
    pub difficulty: U256,
    pub strategy: ConsensusStrategy,
    pub transaction_outputs: ProcessedTransactions,
    pub block_metadata: BlockMetadata,
    pub pruning_point: HashValue,
}

#[derive(Clone)]
pub struct ProcessedTransactions {
    pub included_user_txns: Vec<SignedUserTransaction>,
    pub state_root: HashValue,
    pub txn_accumulator_root: HashValue,
    pub gas_used: u64,
}

pub(crate) struct ProcessTransactionData<P> {
    txn_accumulator: MerkleAccumulator,
    state_db: Arc<ChainStateDB>,
    tx_provider: P,
    user_txns: Vec<SignedUserTransaction>,
    gas_limit: u64,
    gas_used: u64,
    block_meta: BlockMetadata,
    vm_metrics: Option<VMMetrics>,
}

impl<P> ProcessTransactionData<P>
where
    P: TemplateTxProvider + TxPoolSyncService + 'static,
{
    pub fn new(
        storage: Arc<dyn Store>,
        selected_header: HashValue,
        state_db: Arc<ChainStateDB>,
        tx_provider: P,
        user_txns: Vec<SignedUserTransaction>,
        gas_limit: u64,
        gas_used: u64,
        block_meta: BlockMetadata,
        vm_metrics: Option<VMMetrics>,
    ) -> anyhow::Result<Self> {
        let block_info = storage
            .get_block_info(selected_header)?
            .ok_or_else(|| format_err!("Cannot find block info by hash {}", selected_header))?;
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new_with_info(
            txn_accumulator_info.clone(),
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );

        Ok(Self {
            txn_accumulator,
            state_db,
            tx_provider,
            user_txns,
            gas_limit,
            gas_used,
            block_meta,
            vm_metrics,
        })
    }

    /// Run blockmeta first
    fn initialize(&mut self) -> anyhow::Result<(HashValue, HashValue)> {
        let block_metadata_txn = Transaction::BlockMetadata(self.block_meta.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = execute_transactions(
            self.state_db.as_ref(),
            vec![block_metadata_txn],
            self.vm_metrics.clone(),
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");

        match output.status() {
            TransactionStatus::Discard(status) => {
                bail!(
                    "block_metadata txn {:?} is discarded, vm status: {:?}",
                    self.block_meta,
                    status
                );
            }
            TransactionStatus::Keep(_) => {
                let (state_root, txn_accumulator_root) = Self::push_txn_and_state(
                    self.state_db.clone(),
                    &mut self.txn_accumulator,
                    block_meta_txn_hash,
                    output,
                )?;
                Ok((state_root, txn_accumulator_root))
            }
            TransactionStatus::Retry => {
                bail!(
                    "block_metadata txn {:?} is retry impossible",
                    self.block_meta
                );
            }
        }
    }

    pub fn process(mut self) -> anyhow::Result<ProcessedTransactions> {
        let (mut state_root, mut txn_accumulator_root) = self.initialize()?;

        let mut txns = vec![];
        txns.extend(self.user_txns.into_iter().map(Transaction::UserTransaction));

        info!(
            "[BlockProcess] now start to pre execute the transactions {:?}",
            txns.len()
        );
        let txn_outputs = {
            let gas_left = self.gas_limit.checked_sub(self.gas_used).ok_or_else(|| {
                format_err!(
                    "block gas_used {} exceed block gas_limit:{}",
                    self.gas_used,
                    self.gas_limit
                )
            })?;
            execute_block_transactions(
                self.state_db.as_ref(),
                txns.clone(),
                gas_left,
                self.vm_metrics.clone(),
            )?
        };
        info!(
            "[BlockProcess] now end to pre execute the transactions {:?}",
            txn_outputs.len()
        );
        let _untouched_user_txns: Vec<SignedUserTransaction> = if txn_outputs.len() >= txns.len() {
            vec![]
        } else {
            txns.drain(txn_outputs.len()..)
                .map(|t| t.try_into().expect("user txn"))
                .collect()
        };
        debug_assert_eq!(txns.len(), txn_outputs.len());

        let mut included_user_txns: Vec<SignedUserTransaction> = Vec::new();
        let mut discard_txns: Vec<SignedUserTransaction> = Vec::new();

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
                    (state_root, txn_accumulator_root) = Self::push_txn_and_state(
                        self.state_db.clone(),
                        &mut self.txn_accumulator,
                        txn_hash,
                        output,
                    )?;
                    self.gas_used += gas_used;
                    included_user_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus::Retry => {
                    debug!("impossible retry txn {}", txn_hash);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
            };
        }

        for invalid_txn in &discard_txns {
            self.tx_provider.remove_invalid_txn(invalid_txn.id());
        }

        info!(
            "[BlockProcess] txn included: {:?}, discard: {:?}",
            included_user_txns.len(),
            discard_txns.len(),
        );
        Ok(ProcessedTransactions {
            included_user_txns,
            state_root,
            txn_accumulator_root,
            gas_used: self.gas_used,
        })
    }

    fn push_txn_and_state(
        state_db: Arc<ChainStateDB>,
        txn_accumulator: &mut MerkleAccumulator,
        txn_hash: HashValue,
        output: TransactionOutput,
    ) -> anyhow::Result<(HashValue, HashValue)> {
        // Ignore the newly created table_infos.
        // Because they are not needed to calculate state_root, or included to TransactionInfo.
        // This auxiliary function is used to create a new block for mining, nothing need to be persisted to storage.
        let (_table_infos, write_set, events, gas_used, status) = output.into_inner();
        debug_assert!(matches!(status, TransactionStatus::Keep(_)));
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
        state_db
            .apply_write_set(write_set)
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        let txn_state_root = state_db
            .commit()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        let txn_info = TransactionInfo::new(
            txn_hash,
            txn_state_root,
            events.as_slice(),
            gas_used,
            status,
        );
        let accumulator_root = txn_accumulator.append(&[txn_info.id()])?;
        Ok((txn_state_root, accumulator_root))
    }
}
