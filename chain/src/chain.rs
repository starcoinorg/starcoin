// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::repository::{DefaultRepository, Repository};
use anyhow::Result;
use libra_crypto::hash::CryptoHash;
use libra_crypto::HashValue;
use state_view::StateView;
use types::{
    block::{Block, BlockHeader, BlockNumber},
    transaction::TransactionInfo,
};

pub struct Accumulator {}

impl Accumulator {
    pub fn append(&mut self, tx_info: TransactionInfo) -> HashValue {
        unimplemented!()
    }
}

struct Branch {
    block_header: BlockHeader,
    repo: DefaultRepository,
}

impl Branch {
    pub fn set_block_header(&mut self, block_header: BlockHeader) {
        self.block_header = block_header;
    }

    pub fn block_header(&self) -> &BlockHeader {
        &self.block_header
    }

    pub fn repo(&self) -> &dyn Repository {
        &self.repo
    }
}

struct Chain {
    executor: Executor,
    accumulator: Accumulator,
    head: Branch,
    branches: Vec<Branch>,
}

impl Chain {
    pub fn get_block_by_hash(&self, hash: HashValue) -> Block {
        unimplemented!()
    }

    pub fn find_or_fork(&self, header: &BlockHeader) -> Branch {
        unimplemented!()
    }

    pub fn try_connect(&mut self, block: &Block) -> Result<()> {
        let branch = self.find_or_fork(block.header());

        let repo = branch.repo();
        for tx in block.transactions() {
            let output = self.executor.execute_transaction(repo, tx)?;
            let state_root = repo.commit(output.write_set())?;
            let tx_hash = tx.raw_txn().hash();
            let transaction_info =
                TransactionInfo::new(tx_hash, state_root, HashValue::zero(), 0, 0);
            let accumulator_root = self.accumulator.append(transaction_info);
        }
        //todo verify state_root and accumulator_root;
        self.save_block(block);
        repo.flush();
        self.select_head();
        todo!()
    }

    fn select_head(&self) {
        //select head branch;
        todo!()
    }

    fn save_block(&self, block: &Block) {
        todo!()
    }
}
