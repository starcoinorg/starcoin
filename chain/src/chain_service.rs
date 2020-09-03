// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{chain::BlockChain, chain_metrics::CHAIN_METRICS};
use actix::Addr;
use anyhow::{ensure, format_err, Error, Result};
use bus::{Broadcast, BusActor};
use crypto::hash::PlainCryptoHash;
use crypto::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use starcoin_config::NodeConfig;
use starcoin_network::NetworkAsyncService;
use starcoin_network_rpc_api::RemoteChainStateReader;
use starcoin_state_api::{AccountStateReader, ChainStateReader};
use starcoin_statedb::ChainStateDB;
use starcoin_traits::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, ReadableChainService,
    VerifyBlockField,
};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::StartupInfo,
    system_events::{NewBranch, NewHeadBlock},
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
    bus: Addr<BusActor>,
    remote_chain_state: Option<RemoteChainStateReader<NetworkAsyncService>>,
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
        bus: Addr<BusActor>,
        remote_chain_state: Option<RemoteChainStateReader<NetworkAsyncService>>,
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
            bus,
            remote_chain_state,
        })
    }

    pub fn find_or_fork(&self, header: &BlockHeader) -> Result<(bool, Option<BlockChain>)> {
        CHAIN_METRICS.try_connect_count.inc();
        let block_exist = self.block_exist(header.id());
        let block_chain = if self.block_exist(header.parent_hash()) {
            Some(BlockChain::new(
                self.config.net().consensus(),
                header.parent_hash(),
                self.storage.clone(),
            )?)
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
    pub fn get_remote_chain_state(&self) -> Option<RemoteChainStateReader<NetworkAsyncService>> {
        self.remote_chain_state.clone()
    }

    pub fn get_master(&self) -> &BlockChain {
        &self.master
    }

    fn select_head(&mut self, new_branch: BlockChain, repeat_apply: bool) -> Result<()> {
        let block = new_branch.head_block();
        let block_header = block.header().clone();
        let total_difficulty = new_branch.get_total_difficulty()?;
        let broadcast_new_branch = self.is_new_branch(&block_header.parent_hash(), repeat_apply);
        if total_difficulty > self.get_master().get_total_difficulty()? {
            let broadcast_new_master = !self.parent_eq_head(&block_header.parent_hash());
            let (enacted_blocks, retracted_blocks) = if broadcast_new_master {
                self.find_ancestors_from_accumulator(&new_branch)?
            } else {
                (vec![block.clone()], vec![])
            };

            debug_assert!(!enacted_blocks.is_empty());
            debug_assert_eq!(enacted_blocks.last().unwrap(), &block);
            self.update_master(new_branch);
            self.commit_2_txpool(enacted_blocks, retracted_blocks);
            CHAIN_METRICS.broadcast_head_count.inc();
            self.broadcast_new_head(BlockDetail::new(block, total_difficulty));
        } else {
            self.insert_branch(&block_header, repeat_apply);
        }

        if broadcast_new_branch {
            //send new branch event
            self.broadcast_new_branch(block_header);
        }

        CHAIN_METRICS
            .branch_total_count
            .set(self.startup_info.branches.len() as i64);
        self.save_startup()
    }

    fn update_master(&mut self, new_master: BlockChain) {
        let header = new_master.current_header();
        self.master = new_master;
        self.startup_info.update_master(&header);
    }

    fn insert_branch(&mut self, new_block_header: &BlockHeader, repeat_apply: bool) {
        if !repeat_apply
            || self
                .startup_info
                .branch_exist_exclude(&new_block_header.parent_hash())
        {
            self.startup_info.insert_branch(new_block_header);
        }
    }

    fn is_new_branch(&self, parent_id: &HashValue, repeat_apply: bool) -> bool {
        !repeat_apply
            && !self.startup_info.branch_exist_exclude(parent_id)
            && !self.parent_eq_head(parent_id)
    }

    fn parent_eq_head(&self, parent_id: &HashValue) -> bool {
        parent_id == &self.startup_info.master
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
        new_branch: &BlockChain,
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

        ensure!(
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

    fn broadcast_new_head(&self, block: BlockDetail) {
        let bus = self.bus.clone();
        bus.do_send(Broadcast {
            msg: NewHeadBlock(Arc::new(block)),
        });
    }

    fn broadcast_new_branch(&self, maybe_uncle: BlockHeader) {
        let bus = self.bus.clone();
        bus.do_send(Broadcast {
            msg: NewBranch(Arc::new(maybe_uncle)),
        });
    }

    fn verify_uncles(
        &self,
        uncles: &[BlockHeader],
        header: &BlockHeader,
        reader: &dyn ChainStateReader,
    ) -> Result<()> {
        verify_block!(
            VerifyBlockField::Uncle,
            uncles.len() <= MAX_UNCLE_COUNT_PER_BLOCK,
            "too many uncles {} in block {}",
            uncles.len(),
            header.id()
        );
        for uncle in uncles {
            verify_block!(
            VerifyBlockField::Uncle,
            uncle.number < header.number ,
           "uncle block number bigger than or equal to current block ,uncle block number is {} , current block number is {}", uncle.number, header.number
        );
        }

        match header.uncle_hash {
            Some(uncle_hash) => {
                let calculated_hash = HashValue::sha3_256_of(&uncles.to_vec().encode()?);
                verify_block!(
                    VerifyBlockField::Uncle,
                    calculated_hash.eq(&uncle_hash),
                    "uncle hash in header is {},uncle hash calculated is {}",
                    uncle_hash,
                    calculated_hash
                );
            }
            None => {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!("Unexpect uncles, header's uncle hash is None"),
                )
                .into());
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
            if !self.check_common_ancestor(
                uncle.parent_hash,
                epoch_start_number,
                &master_block_headers,
            )? {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!(
                        "can't find ancestor in master uncle id is {:?},epoch start number is {:?}",
                        uncle.id(),
                        header.number
                    ),
                )
                .into());
            }
        }

        let exists_uncles =
            self.merge_exists_uncles(epoch_start_number, self.startup_info.master)?;
        for uncle in uncles {
            if exists_uncles.contains(uncle) {
                debug!("uncle block exists in master,uncle id is {:?}", uncle.id(),);
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!("uncle block exists in master,uncle id is {:?}", uncle.id()),
                )
                .into());
            }
        }

        Ok(())
    }

    fn connect_inner(
        &mut self,
        block: Block,
        execute: bool,
        remote_chain_state: Option<&dyn ChainStateReader>,
    ) -> Result<()> {
        let (block_exist, fork) = self.find_or_fork(block.header())?;
        if block_exist {
            CHAIN_METRICS.duplicate_conn_count.inc();
            self.select_head(fork.expect("Branch not exist."), block_exist)?;
            Err(ConnectBlockError::DuplicateConn(Box::new(block)).into())
        } else if let Some(mut branch) = fork {
            let timer = CHAIN_METRICS
                .exe_block_time
                .with_label_values(&["time"])
                .start_timer();
            if let Some(uncles) = block.uncles() {
                self.verify_uncles(uncles, &block.header, branch.chain_state_reader())?;
            }
            let connected = if execute {
                branch.apply(block.clone())
            } else {
                branch.apply_without_execute(
                    block.clone(),
                    remote_chain_state.expect("remote chain state not set"),
                )
            };
            timer.observe_duration();
            if connected.is_err() {
                debug!("connected failed {:?}", block.header().id());
                CHAIN_METRICS.verify_fail_count.inc();
            } else {
                self.select_head(branch, block_exist)?;
            }
            connected
        } else {
            Err(ConnectBlockError::FutureBlock(Box::new(block)).into())
        }
    }
}

impl<P> ReadableChainService for ChainServiceImpl<P>
where
    P: TxPoolSyncService,
{
    fn try_connect(&mut self, block: Block) -> Result<()> {
        self.connect_inner(block, true, None)
    }

    fn try_connect_without_execute(
        &mut self,
        block: Block,
        remote_chain_state: &dyn ChainStateReader,
    ) -> Result<()> {
        self.connect_inner(block, false, Some(remote_chain_state))
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
                author,
                auth_key_prefix,
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
