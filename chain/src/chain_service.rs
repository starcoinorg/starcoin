// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{chain::BlockChain, chain_metrics::CHAIN_METRICS};
use actix::Addr;
use anyhow::{format_err, Error, Result};
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::hash::PlainCryptoHash;
use crypto::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use starcoin_state_api::{AccountStateReader, ChainStateReader};
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_vm_types::account_config::CORE_CODE_ADDRESS;
use starcoin_vm_types::on_chain_config::{EpochInfo, EpochResource};
use std::collections::HashSet;
use std::sync::Arc;
use storage::Store;
use traits::{ChainReader, ChainService, ChainWriter, ConnectBlockResult, Consensus};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::StartupInfo,
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;

pub struct ChainServiceImpl<C, P>
where
    C: Consensus,
    P: TxPoolSyncService + 'static,
{
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    master: BlockChain<C>,
    storage: Arc<dyn Store>,
    txpool: P,
    bus: Addr<BusActor>,
}

impl<C, P> ChainServiceImpl<C, P>
where
    C: Consensus,
    P: TxPoolSyncService + 'static,
{
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: P,
        bus: Addr<BusActor>,
    ) -> Result<Self> {
        let master = BlockChain::new(config.clone(), startup_info.master, storage.clone())?;
        Ok(Self {
            config,
            startup_info,
            master,
            storage,
            txpool,
            bus,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> Result<(bool, Option<BlockChain<C>>)> {
        CHAIN_METRICS.try_connect_count.inc();
        let block_exist = self.block_exist(header.id());
        let block_chain = if !block_exist {
            if self.block_exist(header.parent_hash()) {
                Some(BlockChain::new(
                    self.config.clone(),
                    header.parent_hash(),
                    self.storage.clone(),
                )?)
            } else {
                None
            }
        } else {
            None
        };
        Ok((block_exist, block_chain))
    }

    pub fn block_exist(&self, block_id: HashValue) -> bool {
        if let Ok(Some(_)) = self.storage.get_block_info(block_id) {
            true
        } else {
            false
        }
    }

    pub fn state_at(&self, _root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    pub fn get_master(&self) -> &BlockChain<C> {
        &self.master
    }

    fn select_head(&mut self, new_branch: BlockChain<C>) -> Result<()> {
        let block = new_branch.head_block();
        let block_header = block.header();
        let total_difficulty = new_branch.get_total_difficulty()?;
        if total_difficulty > self.get_master().get_total_difficulty()? {
            let (enacted_blocks, retracted_blocks) =
                if block.header().parent_hash() == self.startup_info.master {
                    (vec![block.clone()], vec![])
                } else {
                    // TODO: After review the impl of find_common_ancestor in storage.
                    // we can just let find_ancestors do it work, no matter whether fork or not.
                    self.find_ancestors_from_accumulator(&new_branch)?
                };

            debug_assert!(!enacted_blocks.is_empty());
            debug_assert_eq!(enacted_blocks.last().unwrap(), &block);
            self.update_master(new_branch);
            self.commit_2_txpool(enacted_blocks, retracted_blocks);
            CHAIN_METRICS.broadcast_head_count.inc();
            self.broadcast_2_bus(BlockDetail::new(block, total_difficulty));
        } else {
            self.insert_branch(block_header);
        }

        CHAIN_METRICS
            .branch_total_count
            .set(self.startup_info.branches.len() as i64);
        self.save_startup()
    }

    fn update_master(&mut self, new_master: BlockChain<C>) {
        let header = new_master.current_header();
        self.master = new_master;
        self.startup_info.update_master(&header);
    }

    fn insert_branch(&mut self, new_block_header: &BlockHeader) {
        self.startup_info.insert_branch(new_block_header);
    }

    fn save_startup(&self) -> Result<()> {
        let startup_info = self.startup_info.clone();
        self.storage.save_startup_info(startup_info)
    }

    fn commit_2_txpool(&self, enacted: Vec<Block>, retracted: Vec<Block>) {
        if let Err(e) = self.txpool.chain_new_block(enacted, retracted) {
            error!("rollback err : {:?}", e);
        }
    }

    fn find_ancestors_from_accumulator(
        &self,
        new_branch: &BlockChain<C>,
    ) -> Result<(Vec<Block>, Vec<Block>)> {
        let new_header_number = new_branch.current_header().number();
        let master_header_number = self.get_master().current_header().number();
        let mut number = if new_header_number >= master_header_number {
            master_header_number
        } else {
            new_header_number
        };

        let block_enacted = new_branch.current_header().id();
        let block_retracted = self.get_master().current_header().id();

        let mut ancestor = None;
        loop {
            let block_id_1 = new_branch.find_block_by_number(number)?;
            let block_id_2 = self.get_master().find_block_by_number(number)?;

            if block_id_1 == block_id_2 {
                ancestor = Some(block_id_1);
                break;
            }

            if number == 0 {
                break;
            }

            number -= 1;
        }

        assert!(
            ancestor.is_some(),
            "Can not find ancestors from block accumulator."
        );

        let ancestor = ancestor.expect("Ancestor is none.");
        let enacted = self.find_blocks_until(block_enacted, ancestor)?;
        let retracted = self.find_blocks_until(block_retracted, ancestor)?;

        debug!(
            "commit block num:{}, rollback block num:{}",
            enacted.len(),
            retracted.len(),
        );
        Ok((enacted, retracted))
    }

    fn find_blocks_until(&self, from: HashValue, until: HashValue) -> Result<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut tmp = from;
        loop {
            if tmp == until {
                break;
            };
            let block = self
                .storage
                .get_block(tmp)?
                .ok_or_else(|| format_err!("Can not find block {:?}.", tmp))?;
            tmp = block.header().parent_hash();
            blocks.push(block);
        }
        blocks.reverse();

        Ok(blocks)
    }

    fn find_available_uncles(&self, epoch_start_number: BlockNumber) -> Result<Vec<BlockHeader>> {
        let mut exists_uncles =
            self.merge_exists_uncles(epoch_start_number, self.startup_info.master)?;

        let mut uncles = HashSet::new();
        let branchs = self.find_available_branch(epoch_start_number)?;
        for branch_header_id in branchs {
            let available_uncles = self.find_available_uncles_in_branch(
                epoch_start_number,
                branch_header_id,
                &exists_uncles,
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

    fn find_available_branch(&self, epoch_start_number: BlockNumber) -> Result<Vec<HashValue>> {
        let mut result = Vec::new();
        let master_block_headers = self.master_blocks_since(epoch_start_number)?;

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
        let mut id = header_id;

        let mut result = false;
        loop {
            let block_header = self.storage.get_block_header_by_hash(id)?;

            if let Some(block_header) = block_header {
                id = block_header.parent_hash;
                if block_header.number <= epoch_start_number {
                    break;
                }
                if master_block_headers.contains(&block_header) {
                    result = true;
                    break;
                }
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
                    if let Some(uncles) = block.uncles() {
                        for uncle in uncles {
                            if !exists_uncles.contains(uncle) {
                                result.push(uncle.clone());
                            }
                            if result.len() == MAX_UNCLE_COUNT_PER_BLOCK {
                                break;
                            }
                        }
                    }
                    id = block.header.parent_hash;
                }
                None => {
                    break;
                }
            }
        }
        Ok(result)
    }

    pub fn broadcast_2_bus(&self, block: BlockDetail) {
        let bus = self.bus.clone();
        bus.do_send(Broadcast {
            msg: NewHeadBlock(Arc::new(block)),
        });
    }

    fn verify_uncles(
        &self,
        uncles: &[BlockHeader],
        header: &BlockHeader,
        reader: &dyn ChainStateReader,
    ) -> Result<ConnectBlockResult> {
        if uncles.len() > MAX_UNCLE_COUNT_PER_BLOCK {
            debug!("too many uncles {} in block {}", uncles.len(), header.id());
            return Ok(ConnectBlockResult::UncleBlockIllegal);
        }

        for uncle in uncles {
            if uncle.number >= header.number {
                debug!("uncle block number bigger than or equal to current block ,uncle block number is {} , current block number is {}",uncle.number,header.number);
                return Ok(ConnectBlockResult::UncleBlockIllegal);
            }
        }

        match header.uncle_hash {
            Some(uncle_hash) => {
                let calculated_hash = HashValue::sha3_256_of(&uncles.to_vec().encode()?);
                if !calculated_hash.eq(&uncle_hash) {
                    debug!(
                        "uncle hash in header is {},uncle hash calculated is {}",
                        uncle_hash, calculated_hash
                    );
                    return Ok(ConnectBlockResult::UncleBlockIllegal);
                }
            }
            None => {
                return Ok(ConnectBlockResult::UncleBlockIllegal);
            }
        }

        let account_reader = AccountStateReader::new(reader);
        let epoch = account_reader.get_resource::<EpochResource>(CORE_CODE_ADDRESS)?;
        let epoch_start_number = if let Some(epoch) = epoch {
            epoch.start_number()
        } else {
            header.number
        };

        let master_block_headers = self.master_blocks_since(epoch_start_number)?;
        for uncle in uncles {
            if !self.check_common_ancestor(uncle.id(), epoch_start_number, &master_block_headers)? {
                debug!(
                    "can't find ancestor in master uncle id is {:?},epoch start number is {:?}",
                    uncle.id(),
                    header.number
                );
                return Ok(ConnectBlockResult::UncleBlockIllegal);
            }
        }

        let exists_uncles =
            self.merge_exists_uncles(epoch_start_number, self.startup_info.master)?;
        for uncle in uncles {
            if exists_uncles.contains(uncle) {
                debug!("uncle block exists in master,uncle id is {:?}", uncle.id(),);
                return Ok(ConnectBlockResult::DuplicateUncles);
            }
        }

        Ok(ConnectBlockResult::FutureBlock)
    }

    fn connect_inner(&mut self, block: Block, execute: bool) -> Result<ConnectBlockResult> {
        let (block_exist, fork) = self.find_or_fork(block.header())?;
        if block_exist {
            CHAIN_METRICS.duplicate_conn_count.inc();
            Ok(ConnectBlockResult::DuplicateConn)
        } else if let Some(mut branch) = fork {
            let timer = CHAIN_METRICS
                .exe_block_time
                .with_label_values(&["time"])
                .start_timer();
            if let Some(uncles) = block.uncles() {
                if let ConnectBlockResult::VerifyConsensusFailed =
                    self.verify_uncles(uncles, &block.header, branch.chain_state_reader())?
                {
                    return Ok(ConnectBlockResult::VerifyConsensusFailed);
                }
            }
            let connected = if execute {
                branch.apply(block.clone())?
            } else {
                branch.apply_without_execute(block.clone())?
            };
            timer.observe_duration();
            if connected != ConnectBlockResult::SUCCESS {
                debug!("connected failed {:?}", block.header().id());
                CHAIN_METRICS.verify_fail_count.inc();
            } else {
                self.select_head(branch)?;
            }
            Ok(connected)
        } else {
            Ok(ConnectBlockResult::FutureBlock)
        }
    }
}

impl<C, P> ChainService for ChainServiceImpl<C, P>
where
    C: Consensus,
    P: TxPoolSyncService,
{
    fn try_connect(&mut self, block: Block) -> Result<ConnectBlockResult> {
        self.connect_inner(block, true)
    }

    fn try_connect_without_execute(&mut self, block: Block) -> Result<ConnectBlockResult> {
        self.connect_inner(block, false)
    }

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
        self.get_master().get_latest_block_by_uncle(uncle_id, 100)
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

    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self.get_master().current_header().id(),
        };

        if let Ok(Some(block)) = self.get_block_by_hash(block_id) {
            //TODO ensure is need create a new chain?
            let block_chain = self.get_master().new_chain(block_id)?;
            let account_reader = AccountStateReader::new(block_chain.chain_state_reader());
            let epoch = account_reader.get_resource::<EpochResource>(CORE_CODE_ADDRESS)?;
            let epoch_start_number = if let Some(epoch) = epoch {
                epoch.start_number()
            } else {
                block.header.number
            };
            let uncles = self.find_available_uncles(epoch_start_number)?;
            let (block_template, excluded_txns) = block_chain.create_block_template(
                author,
                auth_key_prefix,
                Some(block_id),
                user_txns,
                uncles,
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
