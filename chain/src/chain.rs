// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::Executor;
use crate::repository::{DefaultRepository, Repository};
use anyhow::Result;
use libra_crypto::hash::CryptoHash;
use libra_crypto::HashValue;
use types::{block::Block, transaction::TransactionInfo};

pub struct Accumulator {}

impl Accumulator {
    pub fn append(&mut self, tx_info: TransactionInfo) -> HashValue {
        unimplemented!()
    }
}

struct Chain {
    executor: Executor,
    accumulator: Accumulator,
}

impl Chain {
    pub fn get_block_by_hash(&self, hash: HashValue) -> Block {
        unimplemented!()
    }

    pub fn try_connect(&mut self, block: &Block) -> Result<()> {
        let parent = self.get_block_by_hash(block.header().parent_hash());
        let repo = DefaultRepository::new(parent.header().state_root());
        for tx in block.transactions() {
            self.executor.execute_transaction(&repo, tx);
            repo.commit();
            let state_root = repo.state_root();
            let tx_hash = tx.raw_txn().hash();
            let transaction_info =
                TransactionInfo::new(tx_hash, state_root, HashValue::zero(), 0, 0);
            let accumulator_root = self.accumulator.append(transaction_info);
        }
        self.save_block(block);
        todo!()
    }

    fn save_block(&self, block: &Block) {
        todo!()
    }
}
