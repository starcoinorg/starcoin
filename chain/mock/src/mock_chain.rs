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
use starcoin_storage::Storage;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::block::{BlockNumber, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH};
use starcoin_types::startup_info::ChainInfo;
use std::sync::Arc;

pub struct MockChain {
    net: ChainNetwork,
    head: BlockChain,
    miner: AccountInfo,
    storage: Arc<Storage>,
}

impl MockChain {
    pub fn new(net: ChainNetwork) -> Result<Self> {
        Self::new_with_fork(net, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH)
    }

    pub fn new_with_fork(net: ChainNetwork, fork_number: BlockNumber) -> Result<Self> {
        let (storage, chain_info, _, dag) = Genesis::init_storage_for_mock_test(&net, fork_number)
            .expect("init storage by genesis fail.");

        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            None,
            dag,
        )?;
        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner, storage))
    }

    pub fn new_with_storage(
        net: ChainNetwork,
        storage: Arc<Storage>,
        head_block_hash: HashValue,
        miner: AccountInfo,
        dag: BlockDAG,
    ) -> Result<Self> {
        let chain = BlockChain::new(
            net.time_service(),
            head_block_hash,
            storage.clone(),
            None,
            dag,
        )?;
        Ok(Self::new_inner(net, chain, miner, storage))
    }

    pub fn new_with_chain(
        net: ChainNetwork,
        chain: BlockChain,
        storage: Arc<Storage>,
    ) -> Result<Self> {
        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner, storage))
    }

    fn new_inner(
        net: ChainNetwork,
        head: BlockChain,
        miner: AccountInfo,
        storage: Arc<Storage>,
    ) -> Self {
        Self {
            net,
            head,
            miner,
            storage,
        }
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

    pub fn fork_new_branch(&self, head_id: Option<HashValue>) -> Result<BlockChain> {
        let block_id = match head_id {
            Some(id) => id,
            None => self.head.current_header().id(),
        };
        assert!(self.head.exist_block(block_id)?);
        BlockChain::new(
            self.head.time_service(),
            block_id,
            self.head.get_storage(),
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
            storage: self.storage.clone(),
        })
    }

    pub fn fork_dag(&self, head_id: Option<HashValue>) -> Result<MockChain> {
        let chain = self.fork_new_branch(head_id)?;
        Ok(Self {
            head: chain,
            net: self.net.clone(),
            miner: AccountInfo::random(),
            storage: self.storage.clone(),
        })
    }

    pub fn get_storage(&self) -> Arc<Storage> {
        self.storage.clone()
    }

    pub fn select_head(&mut self, new_block: Block) -> Result<()> {
        //TODO reuse WriteChainService's select_head logic.
        // new block should be execute and save to storage.
        let new_block_id = new_block.id();
        let branch = BlockChain::new(
            self.net.time_service(),
            new_block_id,
            self.head.get_storage(),
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
        let (template, _) = self.head.create_block_template(
            *self.miner.address(),
            None,
            vec![],
            vec![],
            None,
            None,
        )?;
        self.head
            .consensus()
            .create_block(template, self.net.time_service().as_ref())
    }

    pub fn produce_block_by_header(&mut self, parent_header: BlockHeader) -> Result<Block> {
        let (template, _) = self.head.create_block_template_by_header(
            *self.miner.address(),
            parent_header,
            vec![],
            vec![],
            None,
            None,
        )?;
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
        debug!(
            "jacktest: block parent hash: {:?}, number: {:?}",
            block.header().id(),
            block.header().number()
        );
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

    pub fn miner(&self) -> &AccountInfo {
        &self.miner
    }
}
