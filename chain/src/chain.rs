// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::VMConfig;
use executor::TransactionExecutor;
use libra_crypto::hash::CryptoHash;
use libra_crypto::HashValue;
use state_store::StateStore;
use std::marker::PhantomData;
use types::{
    block::{Block, BlockHeader, BlockNumber},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
};

pub struct Accumulator {}

impl Accumulator {
    pub fn append(&mut self, tx_info: TransactionInfo) -> HashValue {
        unimplemented!()
    }
}

struct Branch {
    block_header: BlockHeader,
}

impl Branch {
    pub fn set_block_header(&mut self, block_header: BlockHeader) {
        self.block_header = block_header;
    }

    pub fn block_header(&self) -> &BlockHeader {
        &self.block_header
    }
}

struct Chain<E>
where
    E: TransactionExecutor,
{
    config: VMConfig,
    accumulator: Accumulator,
    head: Branch,
    branches: Vec<Branch>,
    phantom: PhantomData<E>,
}

impl<E> Chain<E>
where
    E: TransactionExecutor,
{
    pub fn get_block_by_hash(&self, hash: HashValue) -> Block {
        unimplemented!()
    }

    pub fn find_or_fork(&self, header: &BlockHeader) -> Branch {
        unimplemented!()
    }

    pub fn state_at(&self, root: HashValue) -> Box<dyn StateStore> {
        unimplemented!()
    }

    //TODO define connect result.
    pub fn try_connect(&mut self, block: Block) -> Result<()> {
        let branch = self.find_or_fork(block.header());

        let store = self.state_at(branch.block_header.state_root());
        let (header, user_txns) = block.clone().into_inner();
        let mut txns = user_txns
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();
        let block_metadata = header.into_metadata();
        txns.push(Transaction::BlockMetadata(block_metadata));
        for txn in txns {
            let txn_hash = txn.hash();
            let output = E::execute_transaction(&self.config, store.as_ref(), txn)?;
            match output.status() {
                TransactionStatus::Discard(status) => return Err(status.clone().into()),
                TransactionStatus::Keep(status) => {
                    //continue.
                }
            }
            let state_root = store.commit()?;
            let transaction_info = TransactionInfo::new(
                txn_hash,
                state_root,
                HashValue::zero(),
                0,
                output.status().vm_status().major_status,
            );
            let accumulator_root = self.accumulator.append(transaction_info);
        }

        //todo verify state_root and accumulator_root;
        self.save_block(block);
        store.flush();
        self.select_head();
        todo!()
    }

    fn select_head(&self) {
        //select head branch;
        todo!()
    }

    fn save_block(&self, block: Block) {
        todo!()
    }
}
