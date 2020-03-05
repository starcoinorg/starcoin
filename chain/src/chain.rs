// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{ChainRequest, ChainResponse};
use actix::prelude::*;
use anyhow::{format_err, Error, Result};
use config::{NodeConfig, VMConfig};
use consensus::{Consensus, ConsensusHeader};
use crypto::{hash::CryptoHash, HashValue};
use executor::TransactionExecutor;
use futures_locks::RwLock;
use logger::prelude::*;
use starcoin_statedb::ChainStateDB;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use traits::{ChainReader, ChainState, ChainStateReader, ChainStateWriter, ChainWriter};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
};

pub struct BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    config: Arc<NodeConfig>,
    //TODO
    //accumulator: Accumulator,
    head: Option<Block>,
    chain_state: ChainStateDB,
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
    storage: Arc<StarcoinStorage>,
}

pub fn load_genesis_block() -> Block {
    let header = BlockHeader::genesis_block_header_for_test();
    Block::new_nil_block_for_test(header)
}

impl<E, C> BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    pub fn new(
        config: Arc<NodeConfig>,
        storage: Arc<StarcoinStorage>,
        head_block_hash: Option<HashValue>,
    ) -> Result<Self> {
        let head = match head_block_hash {
            Some(hash) => Some(
                storage
                    .block_store
                    .get_block_by_hash(hash)?
                    .ok_or(format_err!("Can not find block by hash {}", hash))?,
            ),
            None => None,
        };
        let state_root = head.as_ref().map(|head| head.header().state_root());
        Ok(Self {
            config,
            head,
            chain_state: ChainStateDB::new(storage.clone(), state_root),
            phantom_e: PhantomData,
            phantom_c: PhantomData,
            storage,
        })
    }

    fn save_block(&self, block: &Block) {
        self.storage.block_store.commit_block(block.clone());
        info!("commit block : {:?}", block);
    }

    fn ensure_head(&self) -> &Block {
        self.head
            .as_ref()
            .expect("Must init chain with genesis block first")
    }
}

impl<E, C> ChainReader for BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    fn head_block(&self) -> Block {
        self.ensure_head().clone()
    }

    fn current_header(&self) -> BlockHeader {
        self.ensure_head().header().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.block_store.get_block_header_by_hash(hash)
    }

    fn get_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        self.storage.block_store.get_block_header_by_number(number)
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.storage.block_store.get_block_by_number(number)
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.block_store.get_block_by_hash(hash)
    }

    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>, Error> {
        unimplemented!()
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        unimplemented!()
    }

    fn create_block_template(&self, txns: Vec<SignedUserTransaction>) -> Result<BlockTemplate> {
        let previous_header = self.current_header();
        //TODO execute txns and computer state.
        Ok(BlockTemplate::new(
            previous_header.id(),
            previous_header.number() + 1,
            previous_header.number() + 1,
            AccountAddress::default(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            txns.into(),
        ))
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.chain_state
    }
}

impl<E, C> ChainWriter for BlockChain<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    fn apply(&mut self, block: Block) -> Result<()> {
        let header = block.header();
        let mut is_genesis = false;
        match &self.head {
            Some(head) => {
                //TODO custom verify macro
                assert_eq!(head.header().id(), block.header().parent_hash());
            }
            None => {
                // genesis block
                assert_eq!(block.header().parent_hash(), HashValue::zero());
                is_genesis = true;
            }
        }

        C::verify_header(self, header)?;

        let chain_state = &self.chain_state;
        let mut txns = block
            .transactions()
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();
        let block_metadata = header.clone().into_metadata();

        // remove this after include genesis transaction to genesis block.
        if is_genesis {
            let (_, state_set) = E::init_genesis(&self.config.vm)?;
            txns.push(Transaction::StateSet(state_set));
        } else {
            txns.push(Transaction::BlockMetadata(block_metadata));
        }
        let mut state_root = HashValue::zero();
        for txn in txns {
            let txn_hash = txn.crypto_hash();
            let output = E::execute_transaction(&self.config.vm, chain_state, txn)?;
            match output.status() {
                TransactionStatus::Discard(status) => return Err(status.clone().into()),
                TransactionStatus::Keep(status) => {
                    //continue.
                }
            }
            state_root = chain_state.commit()?;
            let transaction_info = TransactionInfo::new(
                txn_hash,
                state_root,
                HashValue::zero(),
                0,
                output.status().vm_status().major_status,
            );
            //TODO accumulator
            //let accumulator_root = self.accumulator.append(transaction_info);
        }
        assert_eq!(
            block.header().state_root(),
            state_root,
            "verify block:{:?} state_root fail.",
            block.header().id()
        );
        //todo verify  accumulator_root;
        self.save_block(&block);
        chain_state.flush();
        self.head = Some(block);
        //todo
        Ok(())
    }

    fn chain_state(&mut self) -> &dyn ChainState {
        &self.chain_state
    }
}
