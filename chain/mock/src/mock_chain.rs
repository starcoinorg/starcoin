// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_lib::Account;
use starcoin_chain::BlockChain;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_types::block::Block;
use starcoin_vm_types::chain_config::ChainNetwork;

pub struct MockChain {
    head: BlockChain,
    miner: Account,
}

impl MockChain {
    pub fn new(net: ChainNetwork) -> Result<Self> {
        let (storage, startup_info, _) =
            Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

        let chain = BlockChain::new(net, startup_info.master, storage, None)?;
        let miner = Account::random()?;
        Ok(Self { head: chain, miner })
    }

    pub fn head(&self) -> &BlockChain {
        &self.head
    }

    pub fn produce(&self) -> Result<Block> {
        let (template, _) = self.head.create_block_template(
            *self.miner.address(),
            Some(self.miner.authentication_key().prefix().to_vec()),
            None,
            vec![],
            vec![],
            None,
        )?;
        self.head
            .net()
            .consensus()
            .create_block(&self.head, template)
    }

    pub fn apply(&mut self, block: Block) -> Result<()> {
        self.head.apply(block)
    }

    pub fn produce_and_apply(&mut self) -> Result<()> {
        let block = self.produce()?;
        self.apply(block)
    }

    pub fn produce_and_apply_times(&mut self, times: u64) -> Result<()> {
        for _i in 0..times {
            self.produce_and_apply()?;
        }
        Ok(())
    }
}
