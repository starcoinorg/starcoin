// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::BlockChain;
use anyhow::Result;
use crypto::HashValue;
use starcoin_account_api::AccountInfo;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_storage::Storage;
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_types::block::{Block, BlockHeader};
use starcoin_vm_types::genesis_config::ChainNetwork;
use std::sync::Arc;

pub struct MockChain {
    net: ChainNetwork,
    head: BlockChain,
    miner: AccountInfo,
}

impl MockChain {
    pub fn new(net: ChainNetwork) -> Result<Self> {
        let (storage, startup_info, _) =
            Genesis::init_storage_for_test(&net).expect("init storage by genesis fail.");

        let chain = BlockChain::new(net.time_service(), startup_info.master, storage)?;
        let miner = AccountInfo::random();
        Ok(Self {
            net,
            head: chain,
            miner,
        })
    }

    pub fn new_with_storage(
        net: ChainNetwork,
        storage: Arc<Storage>,
        head_block_hash: HashValue,
        miner: AccountInfo,
    ) -> Result<Self> {
        let chain = BlockChain::new(net.time_service(), head_block_hash, storage)?;
        Ok(Self {
            net,
            head: chain,
            miner,
        })
    }

    pub fn net(&self) -> &ChainNetwork {
        &self.net
    }

    pub fn head(&self) -> &BlockChain {
        &self.head
    }

    pub fn fork_new_branch(&self, head_id: Option<HashValue>) -> Result<BlockChain> {
        let block_id = match head_id {
            Some(id) => id,
            None => self.head.current_header().id(),
        };
        assert!(self.head.exist_block(block_id));
        BlockChain::new(self.head.time_service(), block_id, self.head.get_storage())
    }

    pub fn fork(&self, head_id: Option<HashValue>) -> Result<MockChain> {
        let chain = self.fork_new_branch(head_id)?;
        Ok(Self {
            head: chain,
            net: self.net.clone(),
            miner: AccountInfo::random(),
        })
    }

    pub fn produce(&self) -> Result<Block> {
        let (template, _) = self.head.create_block_template(
            *self.miner.address(),
            Some(self.miner.public_key.auth_key()),
            None,
            vec![],
            vec![],
            None,
        )?;
        self.head
            .consensus()
            .create_block(&self.head, template, self.net.time_service().as_ref())
    }

    pub fn apply(&mut self, block: Block) -> Result<()> {
        self.head.apply(block)
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

    pub fn miner(&self) -> &AccountInfo {
        &self.miner
    }
}
