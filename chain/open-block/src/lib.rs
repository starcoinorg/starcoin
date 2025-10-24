// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_chain_api::ExcludedTxns;
use starcoin_crypto::HashValue;
use starcoin_storage::{Store, Store2};
use starcoin_types::block::Version;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::{
    block::{BlockBody, BlockHeader, BlockInfo, BlockTemplate},
    genesis_config::ChainId,
    transaction::SignedUserTransaction,
    U256,
};
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::Arc;

pub struct OpenedBlock {
    previous_block_info: BlockInfo,
    block_meta: BlockMetadata,
    gas_limit: u64,
    gas_used: u64,
    included_user_txns: Vec<SignedUserTransaction>,
    included_user_txns2: Vec<SignedUserTransaction2>,
    uncles: Vec<BlockHeader>,
    chain_id: ChainId,
    difficulty: U256,
    strategy: ConsensusStrategy,
    version: Version,
    pruning_point: HashValue,
    parents_hash: Vec<HashValue>,
    parent_txn_accumulator_root: HashValue,
    parent_state_root: HashValue,
    parent_state_root1: HashValue,
    parent_state_root2: HashValue,
}

impl OpenedBlock {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        storage: Arc<dyn Store>,
        _storage2: Arc<dyn Store2>,
        previous_header: BlockHeader,
        block_gas_limit: u64,
        author: AccountAddress,
        block_timestamp: u64,
        uncles: Vec<BlockHeader>,
        difficulty: U256,
        strategy: ConsensusStrategy,
        tips_hash: Vec<HashValue>,
        version: Version,
        pruning_point: HashValue,
        red_blocks: u64,
    ) -> Result<Self> {
        let previous_block_id = previous_header.id();
        let block_info = storage
            .get_block_info(previous_block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", previous_block_id))?;
        let vm_state_accumulator_info = block_info.get_vm_state_accumulator_info().clone();

        let parent_multi_state = storage.get_vm_multi_state(previous_block_id)?;

        let chain_id = previous_header.chain_id();
        let block_meta = BlockMetadata::new(
            previous_block_id,
            block_timestamp,
            author,
            uncles.len() as u64,
            previous_header.number() + 1,
            chain_id.id().into(),
            previous_header.gas_used(),
            tips_hash.clone(),
            red_blocks,
        );

        Ok(Self {
            previous_block_info: block_info,
            block_meta,
            gas_limit: block_gas_limit,
            gas_used: previous_header.gas_used(),
            included_user_txns: vec![],
            included_user_txns2: vec![],
            uncles,
            chain_id,
            difficulty,
            strategy,
            version,
            pruning_point,
            parents_hash: tips_hash,
            parent_txn_accumulator_root: previous_header.txn_accumulator_root(),
            parent_state_root: vm_state_accumulator_info.accumulator_root,
            parent_state_root1: parent_multi_state.state_root1(),
            parent_state_root2: parent_multi_state.state_root2(),
        })
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    pub fn gas_left(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }

    pub fn block_meta(&self) -> &BlockMetadata {
        &self.block_meta
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn add_transactions(
        &mut self,
        vm1_txns: Vec<SignedUserTransaction>,
        vm2_txns: Vec<SignedUserTransaction2>,
    ) -> ExcludedTxns {
        self.included_user_txns.extend(vm1_txns);
        self.included_user_txns2.extend(vm2_txns);
        ExcludedTxns {
            discarded_txns: Vec::<MultiSignedUserTransaction>::new(),
            untouched_txns: Vec::<MultiSignedUserTransaction>::new(),
        }
    }

    /// Construct a block template for mining。
    pub fn finalize(self) -> Result<BlockTemplate> {
        let uncles = if self.uncles.is_empty() {
            None
        } else {
            Some(self.uncles.clone())
        };
        let body = BlockBody::new_v2(self.included_user_txns, self.included_user_txns2, uncles);
        let block_template = BlockTemplate::new(
            self.previous_block_info
                .block_accumulator_info
                .accumulator_root,
            self.parent_txn_accumulator_root,
            self.parent_state_root,
            self.parent_state_root1,
            self.parent_state_root2,
            self.gas_used,
            body,
            self.chain_id,
            self.difficulty,
            self.strategy,
            self.block_meta.clone(),
            self.version,
            self.pruning_point,
            self.parents_hash.clone(),
        );
        Ok(block_template)
    }
}
