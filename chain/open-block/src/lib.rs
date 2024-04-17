// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_chain_api::ExcludedTxns;
use starcoin_crypto::HashValue;
use starcoin_executor::{execute_block_transactions, execute_transactions, VMMetrics};
use starcoin_force_upgrade::ForceUpgrade;
use starcoin_logger::prelude::*;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Store;
use starcoin_types::account::DEFAULT_EXPIRATION_TIME;
use starcoin_types::block::BlockNumber;
use starcoin_types::genesis_config::{ChainId, ConsensusStrategy};
use starcoin_types::identifier::Identifier;
use starcoin_types::vm_error::KeptVMStatus;
use starcoin_types::{
    account_address::AccountAddress,
    block::{BlockBody, BlockHeader, BlockInfo, BlockTemplate},
    block_metadata::BlockMetadata,
    error::BlockExecutorError,
    transaction::{
        SignedUserTransaction, Transaction, TransactionInfo, TransactionOutput, TransactionStatus,
    },
    U256,
};
use starcoin_vm_runtime::force_upgrade_management::{
    get_force_upgrade_account, get_force_upgrade_block_number,
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::{genesis_address, ModuleUpgradeStrategy};
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::on_chain_config;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::{StateReaderExt, StateView};
use std::{convert::TryInto, sync::Arc};

pub struct OpenedBlock {
    previous_block_info: BlockInfo,
    block_meta: BlockMetadata,
    gas_limit: u64,

    state: ChainStateDB,
    txn_accumulator: MerkleAccumulator,

    gas_used: u64,
    included_user_txns: Vec<SignedUserTransaction>,
    uncles: Vec<BlockHeader>,
    chain_id: ChainId,
    difficulty: U256,
    strategy: ConsensusStrategy,
    vm_metrics: Option<VMMetrics>,
}

impl OpenedBlock {
    pub fn new(
        storage: Arc<dyn Store>,
        previous_header: BlockHeader,
        block_gas_limit: u64,
        author: AccountAddress,
        block_timestamp: u64,
        uncles: Vec<BlockHeader>,
        difficulty: U256,
        strategy: ConsensusStrategy,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let previous_block_id = previous_header.id();
        let block_info = storage
            .get_block_info(previous_block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", previous_block_id))?;
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new_with_info(
            txn_accumulator_info.clone(),
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );

        let chain_state =
            ChainStateDB::new(storage.into_super_arc(), Some(previous_header.state_root()));
        let chain_id = previous_header.chain_id();
        let block_meta = BlockMetadata::new(
            previous_block_id,
            block_timestamp,
            author,
            None,
            uncles.len() as u64,
            previous_header.number() + 1,
            chain_id,
            previous_header.gas_used(),
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
            chain_id,
            difficulty,
            strategy,
            vm_metrics,
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

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    /// Try to add `user_txns` into this block.
    /// Return any txns  not included, either txn is discarded, or block gas limit is reached.
    /// If error occurs during the processing, the `open_block` should be dropped,
    /// as the internal state may be corrupted.
    /// TODO: make the function can be called again even last call returns error.  
    pub fn push_txns(&mut self, user_txns: Vec<SignedUserTransaction>) -> Result<ExcludedTxns> {
        let mut discard_txns: Vec<SignedUserTransaction> = Vec::new();
        let mut txns: Vec<_> = user_txns
            .into_iter()
            .filter(|txn| {
                let is_blacklisted = AddressFilter::is_blacklisted(txn, self.block_number());
                // Discard the txns send by the account in black list after a block number.
                if is_blacklisted {
                    discard_txns.push(txn.clone());
                }
                !is_blacklisted
            })
            .map(Transaction::UserTransaction)
            .collect();

        let txn_outputs = {
            let gas_left = self.gas_limit.checked_sub(self.gas_used).ok_or_else(|| {
                format_err!(
                    "block gas_used {} exceed block gas_limit:{}",
                    self.gas_used,
                    self.gas_limit
                )
            })?;
            execute_block_transactions(
                &self.state,
                txns.clone(),
                gas_left,
                self.vm_metrics.clone(),
            )?
        };

        let untouched_user_txns: Vec<SignedUserTransaction> = if txn_outputs.len() >= txns.len() {
            vec![]
        } else {
            txns.drain(txn_outputs.len()..)
                .map(|t| t.try_into().expect("user txn"))
                .collect()
        };
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
                TransactionStatus::Retry => {
                    debug!("impossible retry txn {}", txn_hash);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
            };
        }

        self.execute_extra_txn()
            .expect("Extra txn must be executed successfully");

        Ok(ExcludedTxns {
            discarded_txns: discard_txns,
            untouched_txns: untouched_user_txns,
        })
    }

    /// Run blockmeta first
    fn initialize(&mut self) -> Result<()> {
        let block_metadata_txn = Transaction::BlockMetadata(self.block_meta.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = execute_transactions(
            &self.state,
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
                let _ = self.push_txn_and_state(block_meta_txn_hash, output)?;
            }
            TransactionStatus::Retry => {
                bail!(
                    "block_metadata txn {:?} is retry impossible",
                    self.block_meta
                );
            }
        };
        Ok(())
    }

    fn push_txn_and_state(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput,
    ) -> Result<(HashValue, HashValue)> {
        self.push_txn_and_state_opt(txn_hash, output, false)
    }

    fn push_txn_and_state_opt(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput,
        is_extra_txn: bool,
    ) -> Result<(HashValue, HashValue)> {
        // Ignore the newly created table_infos.
        // Because they are not needed to calculate state_root, or included to TransactionInfo.
        // This auxiliary function is used to create a new block for mining, nothing need to be persisted to storage.
        let (_table_infos, write_set, events, gas_used, status) = output.into_inner();
        debug_assert!(matches!(status, TransactionStatus::Keep(_)));
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
        self.state
            .apply_write_set(write_set)
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        if is_extra_txn {
            // update stdlib version to 12 directly
            let version_path = on_chain_config::access_path_for_config(
                genesis_address(),
                Identifier::new("Version").unwrap(),
                Identifier::new("Version").unwrap(),
                vec![],
            );
            let version = on_chain_config::Version { major: 12 };
            self.state
                .set(&version_path, bcs_ext::to_bytes(&version)?)?;
        }
        let txn_state_root = self
            .state
            .commit()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        let txn_info = TransactionInfo::new(
            txn_hash,
            txn_state_root,
            events.as_slice(),
            if is_extra_txn { 0 } else { gas_used },
            status,
        );
        let accumulator_root = self.txn_accumulator.append(&[txn_info.id()])?;
        Ok((txn_state_root, accumulator_root))
    }

    /// Construct a block template for mining.
    pub fn finalize(self) -> Result<BlockTemplate> {
        let accumulator_root = self.txn_accumulator.root_hash();
        let state_root = self.state.state_root();
        let uncles = if !self.uncles.is_empty() {
            Some(self.uncles)
        } else {
            None
        };
        let body = BlockBody::new(self.included_user_txns, uncles);
        let block_template = BlockTemplate::new(
            self.previous_block_info
                .block_accumulator_info
                .accumulator_root,
            accumulator_root,
            state_root,
            self.gas_used,
            body,
            self.chain_id,
            self.difficulty,
            self.strategy,
            self.block_meta,
        );
        Ok(block_template)
    }

    /// The logic for handling the forced upgrade will be processed.
    /// First, set the account policy in `0x1::PackageTxnManager` to 100,
    /// Second, after the contract deployment is successful, revert it back.
    fn execute_extra_txn(&mut self) -> Result<()> {
        let extra_txn =
            if self.block_meta.number() == get_force_upgrade_block_number(&self.chain_id) {
                let account = get_force_upgrade_account(&self.chain_id)?;
                let sequence_number = self.state.get_sequence_number(*account.address())?;
                let extra_txn = ForceUpgrade::force_deploy_txn(
                    account,
                    sequence_number,
                    self.block_meta.timestamp() / 1000 + DEFAULT_EXPIRATION_TIME,
                    &self.chain_id,
                )?;
                info!("extra txn in opened block ({:?})", extra_txn.id());
                Transaction::UserTransaction(extra_txn)
            } else {
                return Ok(());
            };
        let extra_txn_hash = extra_txn.id();

        let strategy_path = AccessPath::resource_access_path(
            genesis_address(),
            ModuleUpgradeStrategy::struct_tag(),
        );

        // retrieve old strategy value
        let old_val = self
            .state
            .get_state_value(&StateKey::AccessPath(strategy_path.clone()))?
            .expect("module upgrade strategy should exist");
        // Set strategy to 100 to execute force-deploy-txn directly
        self.state.set(&strategy_path, vec![100])?;

        // execute this special txn without gas limit
        let mut results = execute_transactions(
            &self.state,
            vec![extra_txn.clone()],
            self.vm_metrics.clone(),
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;

        // Restore the old value
        self.state.set(&strategy_path, old_val)?;

        let output = results.pop().expect("executed txn should has output");
        match output.status() {
            TransactionStatus::Discard(status) => {
                bail!(
                    "extra txn {:?} is discarded, vm status: {:?}",
                    extra_txn,
                    status
                );
            }
            TransactionStatus::Keep(_) => {
                // Do not add extra_txn to included_user_txns
                // treat it like BlockMeta txn
                let _ = self.push_txn_and_state_opt(extra_txn_hash, output, true)?;
            }
            TransactionStatus::Retry => {
                bail!("extra txn {:?} is impossible to retry", extra_txn);
            }
        };

        Ok(())
    }
}

pub struct AddressFilter;
//static BLACKLIST: [&str; 0] = [];
impl AddressFilter {
    const ACTIVATION_BLOCK_NUMBER: BlockNumber = 16801958;
    pub fn is_blacklisted(_raw_txn: &SignedUserTransaction, block_number: BlockNumber) -> bool {
        block_number > Self::ACTIVATION_BLOCK_NUMBER
        /*&& BLACKLIST
            .iter()
            .map(|&s| AccountAddress::from_str(s).expect("account address decode must success"))
            .any(|x| x == raw_txn.sender())
        */
    }
}
