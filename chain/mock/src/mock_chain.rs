// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_storage::{Storage, Store};
use starcoin_types::block::{Block, BlockHeader, ExecutedBlock};
use starcoin_types::multi_state::MultiState;
use starcoin_types::startup_info::ChainInfo;
use starcoin_storage::{Storage as Storage2, Store as Store2};
use std::sync::Arc;

pub struct MockChain {
    net: ChainNetwork,
    head: BlockChain,
    miner: AccountInfo,
}

impl MockChain {
    pub fn new(net: ChainNetwork) -> Result<Self> {
        let (storage, storage2, chain_info, _, dag) =
            Genesis::init_storage_for_test(&net).expect("init storage by genesis fail.");

        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage,
            storage2,
            None,
            dag,
        )?;
        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner))
    }

    pub fn new_and_get_storage2(net: ChainNetwork) -> Result<(Self, Arc<Storage2>)> {
        let (storage, storage2, chain_info, _, dag) =
            Genesis::init_storage_for_test(&net).expect("init storage by genesis fail.");

        let storage2_clone = storage2.clone();
        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage,
            storage2,
            None,
            dag,
        )?;
        let miner = AccountInfo::random();
        Ok((Self::new_inner(net, chain, miner), storage2_clone))
    }

    pub fn new_with_storage(
        net: ChainNetwork,
        storage: Arc<Storage>,
        storage2: Arc<Storage2>,
        head_block_hash: HashValue,
        miner: AccountInfo,
        dag: BlockDAG,
    ) -> Result<Self> {
        let chain = BlockChain::new(
            net.time_service(),
            head_block_hash,
            storage,
            storage2,
            None,
            dag,
        )?;
        Ok(Self::new_inner(net, chain, miner))
    }

    pub fn new_with_chain(net: ChainNetwork, chain: BlockChain) -> Result<Self> {
        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner))
    }

    fn new_inner(net: ChainNetwork, head: BlockChain, miner: AccountInfo) -> Self {
        Self { net, head, miner }
    }

    pub fn net(&self) -> &ChainNetwork {
        &self.net
    }

    pub fn head(&self) -> &BlockChain {
        &self.head
    }

    pub fn chain_info(&self) -> ChainInfo {
        self.head.info()
    }

    pub fn get_storage(&self) -> Arc<dyn Store> {
        self.head.get_storage()
    }

    pub fn get_storage2(&self) -> Arc<dyn Store2> {
        self.head.get_storage2()
    }

    pub fn fork_new_branch(&self, head_id: Option<HashValue>) -> Result<BlockChain> {
        let block_id = match head_id {
            Some(id) => id,
            None => self.head.current_header().id(),
        };
        assert!(self.head.has_dag_block(block_id)?);
        BlockChain::new(
            self.head.time_service(),
            block_id,
            self.head.get_storage(),
            self.head.get_storage2(),
            None,
            self.head.dag(),
        )
    }

    pub fn fork(&self, head_id: Option<HashValue>) -> Result<MockChain> {
        let chain = self.fork_new_branch(head_id)?;
        Ok(Self {
            head: chain,
            net: self.net.clone(),
            miner: AccountInfo::random(),
        })
    }

    pub fn select_head(&mut self, new_block: Block) -> Result<()> {
        //TODO reuse WriteChainService's select_head logic.
        // new block should be execute and save to storage.
        let new_block_id = new_block.id();
        let branch = BlockChain::new(
            self.net.time_service(),
            new_block_id,
            self.head.get_storage(),
            self.head.get_storage2(),
            None,
            self.head.dag(),
        )?;
        let branch_total_difficulty = branch.get_total_difficulty()?;
        let head_total_difficulty = self.head.get_total_difficulty()?;
        if branch_total_difficulty > head_total_difficulty {
            self.head = branch;
            debug!("Change to new head: {:?}", self.head.current_header());
            self.net
                .time_service()
                .adjust(new_block.header().timestamp());
        } else {
            debug!(
                "New block({:?})'s total_difficulty({:?}) <= head's total_difficulty({:?})",
                new_block_id, branch_total_difficulty, head_total_difficulty
            );
        }
        Ok(())
    }

    pub fn produce(&self) -> Result<Block> {
        let (template, _) = self
            .head
            .create_block_template_simple(*self.miner.address())?;
        self.head
            .consensus()
            .create_block(template, self.net.time_service().as_ref())
    }

    pub fn apply(&mut self, block: Block) -> Result<()> {
        self.head.apply(block)?;
        Ok(())
    }

    pub fn produce_and_apply(&mut self) -> Result<BlockHeader> {
        let block = self.produce()?;
        let header = block.header().clone();
        self.apply(block)?;
        Ok(header)
    }

    pub fn produce_and_apply_times(&mut self, times: u64) -> Result<()> {
        for _i in 0..times {
            self.produce_and_apply()?;
        }
        Ok(())
    }

    pub fn produce_and_apply_times_for_fork(
        &mut self,
        fork_point: BlockHeader,
        times: u64,
    ) -> Result<BlockHeader> {
        let mut parent_header = fork_point;
        let mut tips = vec![parent_header.id()];
        let mut last = parent_header.clone();
        for _i in 0..times {
            let block_header = self.produce_and_apply_by_tips(parent_header, tips)?;
            parent_header = block_header.clone();
            tips = vec![block_header.id()];
            last = block_header.clone();
        }
        Ok(last)
    }

    pub fn connect(&mut self, executed_block: ExecutedBlock) -> Result<()> {
        self.head.connect(executed_block)?;
        Ok(())
    }

    pub fn produce_and_apply_with_tips_for_times(
        &mut self,
        times: u64,
    ) -> Result<Vec<ExecutedBlock>> {
        let mut blocks = Vec::new();
        for _i in 0..times {
            let header = self.produce_and_apply()?;
            let block = self
                .head
                .get_storage()
                .get_block_by_hash(header.id())?
                .unwrap();
            let block_info = self
                .head
                .get_storage()
                .get_block_info(header.id())?
                .unwrap();
            // Create ExecutedBlock with MultiState
            let executed_block = ExecutedBlock::new(block, block_info, MultiState::default());
            blocks.push(executed_block);
        }
        Ok(blocks)
    }

    pub fn produce_and_apply_by_tips(
        &mut self,
        parent_header: BlockHeader,
        tips: Vec<HashValue>,
    ) -> Result<BlockHeader> {
        let block = self.produce_block_by_tips(parent_header, tips)?;
        let header = block.header().clone();
        self.apply(block)?;
        Ok(header)
    }

    pub fn produce_block_by_tips(
        &mut self,
        parent_header: BlockHeader,
        tips: Vec<HashValue>,
    ) -> Result<Block> {
        let (block_template, _) = self.head.create_block_template(
            *self.miner.address(),
            Some(parent_header), // parent_header
            Vec::new(),          // user_txns
            None,                // uncles
            None,                // block_gas_limit
            Some(tips),          // tips
            HashValue::zero(),   // pruning_point
        )?;
        let new_block = self
            .head
            .consensus()
            .create_block(block_template, self.net.time_service().as_ref())?;
        Ok(new_block)
    }

    pub fn produce_fork_chain(&mut self, one_count: u64, two_count: u64) -> Result<()> {
        let start_header = self.head.current_header();

        let mut parent_one = start_header.clone();
        for _i in 0..one_count {
            let new_block =
                self.produce_block_by_tips(parent_one.clone(), vec![parent_one.id()])?;
            parent_one = new_block.header().clone();
            self.apply(new_block)?;
        }

        let mut parent_two = start_header;
        for _i in 0..two_count {
            let new_block =
                self.produce_block_by_tips(parent_two.clone(), vec![parent_two.id()])?;
            parent_two = new_block.header().clone();
            self.apply(new_block)?;
        }

        // Create a meetup block that has both branches as parents
        let meetup_block = if one_count < two_count {
            self.produce_block_by_tips(parent_two.clone(), vec![parent_one.id(), parent_two.id()])?
        } else {
            self.produce_block_by_tips(parent_one.clone(), vec![parent_one.id(), parent_two.id()])?
        };
        let new_header_id = meetup_block.header().id();
        self.apply(meetup_block)?;

        assert_eq!(self.head.current_header().id(), new_header_id);

        Ok(())
    }

    pub fn miner(&self) -> &AccountInfo {
        &self.miner
    }
}
