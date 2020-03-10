// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{ChainRequest, ChainResponse};
use actix::prelude::*;
use anyhow::{ensure, format_err, Error, Result};
use config::{NodeConfig, VMConfig};
use consensus::{Consensus, ConsensusHeader};
use crypto::{hash::CryptoHash, HashValue};
use executor::mock_executor::mock_mint_txn;
use executor::TransactionExecutor;
use futures_locks::RwLock;
use logger::prelude::*;
use starcoin_accumulator::node_index::NodeIndex;
use starcoin_accumulator::{Accumulator, AccumulatorNodeStore, MerkleAccumulator};
use starcoin_statedb::ChainStateDB;
use state_tree::StateNodeStore;
use std::cell::RefCell;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, BlockStorageOp, StarcoinStorage};
use traits::{
    ChainReader, ChainState, ChainStateReader, ChainStateWriter, ChainWriter, TxPoolAsyncService,
};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    block_metadata::BlockMetadata,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
};

pub struct BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: StateNodeStore + BlockStorageOp + AccumulatorNodeStore + 'static,
    P: TxPoolAsyncService + 'static,
{
    config: Arc<NodeConfig>,
    //TODO
    accumulator: MerkleAccumulator,
    head: Option<Block>,
    chain_state: ChainStateDB,
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
    storage: Arc<S>,
    txpool: P,
}

pub fn load_genesis_block() -> Block {
    let header = BlockHeader::genesis_block_header_for_test();
    Block::new_nil_block_for_test(header)
}

impl<E, C, S, P> BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: StateNodeStore + BlockStorageOp + AccumulatorNodeStore,
    P: TxPoolAsyncService,
{
    pub fn new(
        config: Arc<NodeConfig>,
        storage: Arc<S>,
        head_block_hash: Option<HashValue>,
        txpool: P,
    ) -> Result<Self> {
        let head = match head_block_hash {
            Some(hash) => Some(
                storage
                    .get_block_by_hash(hash)?
                    .ok_or(format_err!("Can not find block by hash {}", hash))?,
            ),
            None => None,
        };
        let is_genesis = head.is_none();
        let state_root = head.as_ref().map(|head| head.header().state_root());
        let mut chain = Self {
            config: config.clone(),
            accumulator: MerkleAccumulator::new(vec![], 0, 0, storage.clone()).unwrap(),
            head,
            chain_state: ChainStateDB::new(storage.clone(), state_root),
            phantom_e: PhantomData,
            phantom_c: PhantomData,
            storage,
            txpool,
        };
        if is_genesis {
            ///init genesis block
            //TODO should process at here ?
            let (state_root, chain_state_set) = E::init_genesis(&config.vm)?;
            let genesis_block =
                Block::genesis_block(HashValue::zero(), state_root, chain_state_set);
            info!("Init with genesis block: {:?}", genesis_block);
            chain.apply(genesis_block)?;
        }
        Ok(chain)
    }

    fn verify_proof(
        &self,
        expect_root: HashValue,
        leaves: &[HashValue],
        first_leaf_idx: u64,
    ) -> Result<()> {
        ensure!(leaves.len() > 0, "invalid leaves.");
        leaves.iter().enumerate().for_each(|(i, hash)| {
            let leaf_index = first_leaf_idx + i as u64;
            let proof = self.accumulator.get_proof(leaf_index).unwrap().unwrap();
            proof.verify(expect_root, *hash, leaf_index).unwrap();
        });
        Ok(())
    }

    fn save_block(&self, block: &Block) {
        self.storage.commit_block(block.clone());
        info!("commit block : {:?}", block);
    }

    fn ensure_head(&self) -> &Block {
        self.head
            .as_ref()
            .expect("Must init chain with genesis block first")
    }

    fn gen_tx_for_test(&self) {
        let tx = mock_mint_txn(AccountAddress::random(), 100);
        // info!("gen test txn: {:?}", tx);
        let txpool = self.txpool.clone();
        Arbiter::spawn(async move {
            info!("gen_tx_for_test call txpool.");
            txpool.add(tx.try_into().unwrap()).await.unwrap();
        });
    }
}

impl<E, C, S, P> ChainReader for BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: StateNodeStore + BlockStorageOp + AccumulatorNodeStore,
    P: TxPoolAsyncService,
{
    fn head_block(&self) -> Block {
        self.ensure_head().clone()
    }

    fn current_header(&self) -> BlockHeader {
        self.ensure_head().header().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn get_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_number(number)
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.storage.get_block_by_number(number)
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>, Error> {
        unimplemented!()
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        unimplemented!()
    }

    fn create_block_template(
        &self,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let previous_header = self.current_header();

        //TODO read address from config
        let author = AccountAddress::random();
        //TODO calculate gas limit etc.
        let mut txns = user_txns
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();

        //TODO refactor BlockMetadata to Coinbase transaction.
        txns.push(Transaction::BlockMetadata(BlockMetadata::new(
            HashValue::zero(),
            0,
            author,
        )));
        let chain_state =
            ChainStateDB::new(self.storage.clone(), Some(previous_header.state_root()));
        let mut state_root = HashValue::zero();
        let mut transaction_hash = vec![];
        for txn in txns {
            let txn_hash = txn.crypto_hash();
            let output = E::execute_transaction(&self.config.vm, &chain_state, txn)?;
            match output.status() {
                TransactionStatus::Discard(status) => return Err(status.clone().into()),
                TransactionStatus::Keep(status) => {
                    //continue.
                }
            }
            //TODO should not commit here.
            state_root = chain_state.commit()?;
            let transaction_info = TransactionInfo::new(
                txn_hash,
                state_root,
                HashValue::zero(),
                0,
                output.status().vm_status().major_status,
            );
            transaction_hash.push(txn_hash);
        }

        //TODO accumulator
        let (accumulator_root, _) = self.accumulator.append(&transaction_hash).unwrap();

        //TODO execute txns and computer state.
        Ok(BlockTemplate::new(
            previous_header.id(),
            previous_header.number() + 1,
            previous_header.number() + 1,
            author,
            accumulator_root,
            state_root,
            0,
            0,
            user_txns.into(),
        ))
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.chain_state
    }

    fn gen_tx(&self) -> Result<()> {
        self.gen_tx_for_test();
        Ok(())
    }
}

impl<E, C, S, P> ChainWriter for BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: StateNodeStore + BlockStorageOp + AccumulatorNodeStore,
    P: TxPoolAsyncService,
{
    fn apply(&mut self, block: Block) -> Result<()> {
        let header = block.header();
        let mut is_genesis = false;
        match &self.head {
            Some(head) => {
                info!("Apply block {:?} to {:?}", block.header(), head.header());
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
        let mut transaction_hash = vec![];
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
            transaction_hash.push(txn_hash);
        }

        let (accumulator_root, first_leaf_idx) =
            self.accumulator.append(&transaction_hash).unwrap();
        assert_eq!(
            block.header().state_root(),
            state_root,
            "verify block:{:?} state_root fail.",
            block.header().id()
        );
        //todo verify  accumulator_root;
        self.verify_proof(accumulator_root, &transaction_hash, first_leaf_idx);
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
