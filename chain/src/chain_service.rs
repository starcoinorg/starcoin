// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use anyhow::{format_err, Error, Result};
use crypto::ed25519::Ed25519PublicKey;
use crypto::hash::PlainCryptoHash;
use crypto::HashValue;
use logger::prelude::*;
use starcoin_config::NodeConfig;
use starcoin_state_api::AccountStateReader;
use starcoin_traits::{ChainReader, ReadableChainService};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::StartupInfo,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};
use starcoin_vm_types::account_config::CORE_CODE_ADDRESS;
use starcoin_vm_types::on_chain_config::{EpochInfo, EpochResource, GlobalTimeOnChain};
use std::collections::HashSet;
use std::iter::Iterator;
use std::sync::Arc;
use storage::Store;

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;

pub struct ChainServiceImpl<P>
where
    P: TxPoolSyncService + 'static,
{
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    master: BlockChain,
    storage: Arc<dyn Store>,
    txpool: P,
}

impl<P> ChainServiceImpl<P>
where
    P: TxPoolSyncService + 'static,
{
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: P,
    ) -> Result<Self> {
        let master = BlockChain::new(
            config.net().consensus(),
            startup_info.master,
            storage.clone(),
        )?;
        Ok(Self {
            config,
            startup_info,
            master,
            storage,
            txpool,
        })
    }

    pub fn get_master(&self) -> &BlockChain {
        &self.master
    }

    pub fn update_chain_head(&mut self, block: Block) -> Result<()> {
        self.master.update_chain_head(block)
    }

    pub fn switch_master(&mut self, new_head_id: HashValue) -> Result<()> {
        let old_head_id = self.get_master().current_header().id();
        if old_head_id != new_head_id {
            let old_difficulty = self
                .storage
                .get_block_info(old_head_id)?
                .ok_or_else(|| {
                    format_err!("block info not exist by old block id {:?}.", old_head_id)
                })?
                .get_total_difficulty();
            let new_difficulty = self
                .storage
                .get_block_info(new_head_id)?
                .ok_or_else(|| {
                    format_err!("block info not exist by new block id {:?}.", new_head_id)
                })?
                .get_total_difficulty();
            assert!(new_difficulty > old_difficulty);
            self.master = BlockChain::new(
                self.config.net().consensus(),
                new_head_id,
                self.storage.clone(),
            )?;
        }
        Ok(())
    }

    fn find_available_uncles(&self, epoch_start_number: BlockNumber) -> Result<Vec<BlockHeader>> {
        let mut exists_uncles =
            self.merge_exists_uncles(epoch_start_number, self.startup_info.master)?;

        let master_block_headers = self.master_blocks_since(epoch_start_number)?;

        let mut uncles = HashSet::new();
        let branches = self.find_available_branch(epoch_start_number, &master_block_headers)?;

        let master_block_header_ids = master_block_headers
            .iter()
            .map(|header| header.id())
            .collect();
        for branch_header_id in branches {
            if uncles.len() == MAX_UNCLE_COUNT_PER_BLOCK {
                break;
            }

            let available_uncles = self.find_available_uncles_in_branch(
                epoch_start_number,
                branch_header_id,
                &exists_uncles,
                &master_block_header_ids,
            )?;
            for uncle in available_uncles {
                if !uncles.contains(&uncle) {
                    uncles.insert(uncle.clone());
                    exists_uncles.insert(uncle);
                }
                if uncles.len() == MAX_UNCLE_COUNT_PER_BLOCK {
                    break;
                }
            }
        }
        Ok(uncles.into_iter().collect())
    }

    fn find_available_branch(
        &self,
        epoch_start_number: BlockNumber,
        master_block_headers: &HashSet<BlockHeader>,
    ) -> Result<Vec<HashValue>> {
        let mut result = Vec::new();

        for branch_header_id in &self.startup_info.branches {
            if self.check_common_ancestor(
                *branch_header_id,
                epoch_start_number,
                &master_block_headers,
            )? {
                result.push(*branch_header_id)
            }
        }
        Ok(result)
    }

    fn master_blocks_since(&self, epoch_start_number: BlockNumber) -> Result<HashSet<BlockHeader>> {
        let mut result = HashSet::new();

        let mut id = self.startup_info.master;

        loop {
            let block_header = self.storage.get_block_header_by_hash(id)?;

            if let Some(block_header) = block_header {
                id = block_header.parent_hash;
                if block_header.number <= epoch_start_number {
                    break;
                }
                result.insert(block_header);
            }
        }
        Ok(result)
    }

    fn check_common_ancestor(
        &self,
        header_id: HashValue,
        epoch_start_number: BlockNumber,
        master_block_headers: &HashSet<BlockHeader>,
    ) -> Result<bool> {
        let mut result = false;
        let block_header = self.storage.get_block_header_by_hash(header_id)?;

        if let Some(block_header) = block_header {
            if master_block_headers.contains(&block_header)
                && block_header.number < epoch_start_number
            {
                result = true;
            }
        }
        Ok(result)
    }

    fn merge_exists_uncles(
        &self,
        epoch_start_number: BlockNumber,
        block_id: HashValue,
    ) -> Result<HashSet<BlockHeader>> {
        let mut exists_uncles = HashSet::new();

        let mut id = block_id;
        loop {
            let block = self.storage.get_block_by_hash(id)?;
            match block {
                Some(block) => {
                    if block.header.number < epoch_start_number {
                        break;
                    }
                    if let Some(uncles) = block.uncles() {
                        for uncle in uncles {
                            exists_uncles.insert(uncle.clone());
                        }
                    }
                    id = block.header.parent_hash;
                }
                None => {
                    break;
                }
            }
        }
        Ok(exists_uncles)
    }

    fn find_available_uncles_in_branch(
        &self,
        epoch_start_number: BlockNumber,
        block_id: HashValue,
        exists_uncles: &HashSet<BlockHeader>,
        master_block_headers: &HashSet<HashValue>,
    ) -> Result<Vec<BlockHeader>> {
        let mut id = block_id;
        let mut result = vec![];

        loop {
            let block = self.storage.get_block_by_hash(id)?;

            match block {
                Some(block) => {
                    if block.header.number < epoch_start_number {
                        break;
                    }
                    if !exists_uncles.contains(block.header())
                        && master_block_headers.contains(&block.header.parent_hash)
                    {
                        result.push(block.header().clone());
                    }
                    if result.len() == MAX_UNCLE_COUNT_PER_BLOCK {
                        break;
                    }
                    id = block.header().parent_hash();
                }
                None => {
                    break;
                }
            }
        }
        Ok(result)
    }
}

impl<P> ReadableChainService for ChainServiceImpl<P>
where
    P: TxPoolSyncService,
{
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn get_block_state_by_hash(&self, hash: HashValue) -> Result<Option<BlockState>> {
        self.storage.get_block_state(hash)
    }

    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>> {
        self.storage.get_block_info(hash)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>, Error> {
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        self.get_master().get_transaction_info(txn_hash)
    }

    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>, Error> {
        self.storage.get_block_transaction_infos(block_id)
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>, Error> {
        self.storage
            .get_transaction_info_by_block_and_index(block_id, idx)
    }
    fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>, Error> {
        self.storage.get_contract_events(txn_info_id)
    }

    fn master_head_header(&self) -> BlockHeader {
        self.get_master().current_header()
    }

    fn master_head_block(&self) -> Block {
        self.get_master().head_block()
    }

    fn master_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.get_master().get_block_by_number(number)
    }

    fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>> {
        self.get_master().get_latest_block_by_uncle(uncle_id, 500)
    }

    fn master_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        self.get_master().get_header_by_number(number)
    }
    fn master_startup_info(&self) -> StartupInfo {
        self.startup_info.clone()
    }

    fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>> {
        self.get_master().get_blocks_by_number(number, count)
    }

    fn epoch_info(&self) -> Result<EpochInfo> {
        self.get_master().epoch_info()
    }

    fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo> {
        self.get_master().get_epoch_info_by_number(Some(number))
    }

    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain> {
        self.get_master().get_global_time_by_number(number)
    }

    fn create_block_template(
        &self,
        author_public_key: Ed25519PublicKey,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self.get_master().current_header().id(),
        };

        if let Ok(Some(block)) = self.get_block_by_hash(block_id) {
            let block_chain = BlockChain::new(
                self.config.net().consensus(),
                block.id(),
                self.storage.clone(),
            )?;
            let account_reader = AccountStateReader::new(block_chain.chain_state_reader());
            let epoch = account_reader.get_resource::<EpochResource>(CORE_CODE_ADDRESS)?;
            let epoch_start_number = if let Some(epoch) = epoch {
                epoch.start_number()
            } else {
                block.header.number
            };
            let uncles = self.find_available_uncles(epoch_start_number)?;
            debug!("uncles len: {}", uncles.len());
            let (block_template, excluded_txns) = block_chain.create_block_template(
                author_public_key,
                Some(block_id),
                user_txns,
                uncles,
                self.config.miner.block_gas_limit,
            )?;
            // remove invalid txn from txpool
            for invalid_txn in excluded_txns.discarded_txns {
                let _ = self.txpool.remove_txn(invalid_txn.crypto_hash(), true);
            }

            Ok(block_template)
        } else {
            Err(format_err!("Block {:?} not exist.", block_id))
        }
    }
}
