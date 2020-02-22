// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::starcoin_chain_state::StarcoinChainState;
use crate::ChainWriter;
use actix::prelude::*;
use anyhow::Result;
use chain_state::ChainState;
use config::VMConfig;
use consensus::{ChainReader, Consensus, ConsensusHeader};
use crypto::{hash::CryptoHash, HashValue};
use executor::TransactionExecutor;
use std::marker::PhantomData;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
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

struct BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    config: VMConfig,
    accumulator: Accumulator,
    head: Branch,
    branches: Vec<Branch>,
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
}

impl<E, C> ChainReader for BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    fn current_header(&self) -> BlockHeader {
        unimplemented!()
    }

    fn get_header_by_hash(&self, hash: HashValue) -> BlockHeader {
        unimplemented!()
    }

    fn head_block(&self) -> Block {
        unimplemented!()
    }

    fn get_header_by_number(&self, number: u64) -> BlockHeader {
        unimplemented!()
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Block {
        unimplemented!()
    }
    fn get_block_by_hash(&self, hash: HashValue) -> Option<Block> {
        unimplemented!()
    }

    fn create_block_template(&self) -> Result<BlockTemplate> {
        let previous_header = self.current_header();
        let header = BlockHeader::new(
            previous_header.id(),
            previous_header.number() + 1,
            0,
            AccountAddress::default(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            vec![],
        );
        // get pending tx from pool, and execute to build BlockTemplate.
        todo!()
    }
}

impl<E, C> BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    pub fn find_or_fork(&self, header: &BlockHeader) -> Branch {
        unimplemented!()
    }

    pub fn state_at(&self, root: HashValue) -> StarcoinChainState {
        unimplemented!()
    }

    fn select_head(&self) {
        //select head branch;
        todo!()
    }

    fn save_block(&self, block: &Block) {
        todo!()
    }
}

impl<E, C> ChainWriter for BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    //TODO define connect result.
    fn try_connect(&mut self, block: Block) -> Result<()> {
        let header = block.header();
        let branch = self.find_or_fork(&header);
        C::verify_header(self, header)?;
        let chain_state = self.state_at(branch.block_header.state_root());
        let mut txns = block
            .transactions()
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();
        let block_metadata = header.clone().into_metadata();
        txns.push(Transaction::BlockMetadata(block_metadata));
        for txn in txns {
            let txn_hash = txn.crypto_hash();
            let output = E::execute_transaction(&self.config, &chain_state, txn)?;
            match output.status() {
                TransactionStatus::Discard(status) => return Err(status.clone().into()),
                TransactionStatus::Keep(status) => {
                    //continue.
                }
            }
            let state_root = chain_state.commit()?;
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
        self.save_block(&block);
        chain_state.flush();
        self.select_head();
        todo!()
    }
}
