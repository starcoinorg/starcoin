// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_service::BlockChainCollection;
use actix::prelude::*;
use anyhow::{ensure, format_err, Error, Result};
use config::NodeConfig;
use crypto::{hash::CryptoHash, HashValue};
use executor::executor::mock_create_account_txn;
use executor::TransactionExecutor;
use logger::prelude::*;
use once_cell::sync::Lazy;
use starcoin_accumulator::node::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::{ChainState, ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::TxPoolAsyncService;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use storage::Store;
use traits::Consensus;
use traits::{ChainReader, ChainWriter};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate, BLOCK_INFO_DEFAULT_ID},
    block_metadata::BlockMetadata,
    startup_info::ChainInfo,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
    U256, U512,
};

pub static DEFAULT_BLOCK_INFO: Lazy<BlockInfo> = Lazy::new(|| {
    BlockInfo::new(
        *BLOCK_INFO_DEFAULT_ID,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        vec![],
        0,
        0,
        U512::zero(),
    )
});

pub struct BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: Store + 'static,
    P: TxPoolAsyncService + 'static,
{
    pub config: Arc<NodeConfig>,
    //TODO
    accumulator: MerkleAccumulator,
    head: Block,
    chain_state: ChainStateDB,
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
    pub storage: Arc<S>,
    pub txpool: P,
    chain_info: ChainInfo,
    pub block_chain_collection: Arc<BlockChainCollection<E, C, S, P>>,
}

impl<E, C, S, P> BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: Store,
    P: TxPoolAsyncService,
{
    pub fn new(
        config: Arc<NodeConfig>,
        chain_info: ChainInfo,
        storage: Arc<S>,
        txpool: P,
        block_chain_collection: Arc<BlockChainCollection<E, C, S, P>>,
    ) -> Result<Self> {
        let head_block_hash = chain_info.get_head();
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or(format_err!(
                "Can not find block by hash {}",
                head_block_hash
            ))?;
        let block_info = match storage.clone().get_block_info(head_block_hash) {
            Ok(Some(block_info_1)) => block_info_1,
            Err(e) => {
                warn!("err : {:?}", e);
                DEFAULT_BLOCK_INFO.clone()
            }
            _ => DEFAULT_BLOCK_INFO.clone(),
        };

        let state_root = head.header().state_root();
        let chain = Self {
            config: config.clone(),
            accumulator: MerkleAccumulator::new(
                chain_info.branch_id(),
                block_info.accumulator_root,
                block_info.frozen_subtree_roots,
                block_info.num_leaves,
                block_info.num_nodes,
                storage.clone(),
            )
            .unwrap(),
            head,
            chain_state: ChainStateDB::new(storage.clone(), Some(state_root)),
            phantom_e: PhantomData,
            phantom_c: PhantomData,
            storage,
            txpool,
            chain_info,
            block_chain_collection,
        };
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

    pub fn save_block(&self, block: &Block) {
        if let Err(e) = self
            .storage
            .commit_branch_block(self.get_chain_info().branch_id(), block.clone())
        {
            warn!("err : {:?}", e);
        }
        info!("commit block : {:?}", block.header().id());
    }

    fn get_block_info(&self, block_id: HashValue) -> BlockInfo {
        let block_info = match self.storage.get_block_info(block_id) {
            Ok(Some(block_info_1)) => block_info_1,
            Err(e) => {
                warn!("err : {:?}", e);
                DEFAULT_BLOCK_INFO.clone()
            }
            _ => DEFAULT_BLOCK_INFO.clone(),
        };
        block_info
    }
    pub fn save_block_info(&self, block_info: BlockInfo) {
        if let Err(e) = self.storage.save_block_info(block_info) {
            warn!("err : {:?}", e);
        }
    }

    fn gen_tx_for_test(&self) {
        let tx = mock_create_account_txn();
        // info!("gen test txn: {:?}", tx);
        let txpool = self.txpool.clone();
        Arbiter::spawn(async move {
            info!("gen_tx_for_test call txpool.");
            txpool.add(tx.try_into().unwrap()).await.unwrap();
        });
    }

    pub fn latest_blocks(&self, size: u64) {
        let mut count = 0;
        let mut last = self.head.header().clone();
        loop {
            info!(
                "block chain :: number : {} , block_id : {:?}",
                last.number(),
                last.id()
            );
            if last.number() == 0 || count >= size {
                break;
            }
            last = self
                .get_header(last.parent_hash())
                .unwrap()
                .unwrap()
                .clone();
            count = count + 1;
        }
    }

    pub fn create_block_template_inner(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        previous_header: BlockHeader,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        //TODO calculate gas limit etc.
        let mut txns = user_txns
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        txns.push(Transaction::BlockMetadata(BlockMetadata::new(
            previous_header.id(),
            timestamp,
            author,
            auth_key_prefix.clone(),
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
                TransactionStatus::Keep(_status) => {
                    //continue.
                }
            }
            //TODO should not commit here.
            state_root = chain_state.commit()?;
            let _transaction_info = TransactionInfo::new(
                txn_hash,
                state_root,
                HashValue::zero(),
                0,
                output.status().vm_status().major_status,
            );
            transaction_hash.push(txn_hash);
        }

        let block_info = self.get_block_info(previous_header.id());
        let accumulator = MerkleAccumulator::new(
            self.chain_info.branch_id(),
            block_info.accumulator_root,
            block_info.frozen_subtree_roots,
            block_info.num_leaves,
            block_info.num_nodes,
            self.storage.clone(),
        )
        .unwrap();
        let (accumulator_root, first_leaf_idx) = accumulator.append(&transaction_hash).unwrap();
        //Fixme proof verify
        transaction_hash.iter().enumerate().for_each(|(i, hash)| {
            let leaf_index = first_leaf_idx + i as u64;
            let proof = accumulator.get_proof(leaf_index).unwrap().unwrap();
            proof.verify(accumulator_root, *hash, leaf_index).unwrap();
        });
        //TODO execute txns and computer state.

        Ok(BlockTemplate::new(
            previous_header.id(),
            timestamp,
            previous_header.number() + 1,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            0,
            0,
            difficulty,
            user_txns.into(),
        ))
    }

    pub fn fork(&self, block_header: &BlockHeader) -> Option<ChainInfo> {
        if self.exist_block(block_header.parent_hash()) {
            Some(if self.head.header().id() == block_header.parent_hash() {
                self.get_chain_info()
            } else {
                ChainInfo::new(
                    Some(self.get_chain_info().branch_id()),
                    block_header.parent_hash(),
                    block_header,
                )
            })
        } else {
            None
        }
    }

    pub fn get_branch_id(&self, number: BlockNumber) -> Option<HashValue> {
        self.block_chain_collection
            .get_branch_id(&self.chain_info.branch_id(), number)
    }

    pub fn update_head(&mut self, latest_block: BlockHeader) {
        self.chain_info.update_head(latest_block)
    }
}

impl<E, C, S, P> ChainReader for BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: Store,
    P: TxPoolAsyncService,
{
    fn head_block(&self) -> Block {
        self.head.clone()
    }

    fn current_header(&self) -> BlockHeader {
        self.head.header().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        assert!(self.exist_block(hash));
        Ok(Some(
            self.get_block(hash).unwrap().unwrap().header().clone(),
        ))
    }

    fn get_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        if let Some(branch_id) = self.get_branch_id(number) {
            self.storage.get_header_by_branch_number(branch_id, number)
        } else {
            Ok(None)
        }
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        if let Some(branch_id) = self.get_branch_id(number) {
            self.storage.get_block_by_branch_number(branch_id, number)
        } else {
            warn!("branch id not found.");
            Ok(None)
        }
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        let block = self.storage.get_block_by_hash(hash);
        match block {
            Ok(tmp) => match tmp {
                Some(b) => {
                    if let Ok(Some(block_header)) = self.get_header_by_number(b.header().number()) {
                        if block_header.id() == b.header().id() {
                            return Ok(Some(b));
                        } else {
                            warn!("block is miss match {:?} : {:?}", hash, block_header.id());
                        }
                    }
                }
                None => {
                    warn!("Get block from storage return none.");
                }
            },
            Err(e) => {
                warn!("err:{:?}", e);
            }
        }

        return Ok(None);
    }

    fn get_transaction(&self, _hash: HashValue) -> Result<Option<Transaction>, Error> {
        unimplemented!()
    }

    fn get_transaction_info(&self, _hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        unimplemented!()
    }

    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self.current_header().id(),
        };
        assert!(self.exist_block(block_id));
        let previous_header = self.get_header(block_id).unwrap().unwrap();
        self.create_block_template_inner(
            author,
            auth_key_prefix,
            previous_header,
            difficulty,
            user_txns,
        )
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.chain_state
    }

    fn gen_tx(&self) -> Result<()> {
        self.gen_tx_for_test();
        Ok(())
    }

    fn get_chain_info(&self) -> ChainInfo {
        self.chain_info.clone()
    }

    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>> {
        let id = match block_id {
            Some(hash) => hash,
            None => self.current_header().id(),
        };
        assert!(self.exist_block(id));
        self.storage.get_block_info(id)
    }

    fn get_total_difficulty(&self) -> Result<U512> {
        let block_info = self.storage.get_block_info(self.head.header().id())?;
        Ok(block_info.map_or(U512::zero(), |info| info.total_difficulty))
    }

    fn exist_block(&self, block_id: HashValue) -> bool {
        if let Ok(Some(_)) = self.get_block(block_id) {
            true
        } else {
            false
        }
    }
}

impl<E, C, S, P> ChainWriter for BlockChain<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    S: Store,
    P: TxPoolAsyncService,
{
    fn apply(&mut self, block: Block) -> Result<()> {
        let header = block.header();
        info!(
            "Apply block {:?} to {:?}",
            block.header(),
            self.head.header()
        );
        //TODO custom verify macro
        assert_eq!(self.head.header().id(), block.header().parent_hash());

        C::verify_header(self.config.clone(), self, header)?;

        let chain_state = &self.chain_state;
        let mut txns = block
            .transactions()
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();
        let block_metadata = header.clone().into_metadata();

        txns.push(Transaction::BlockMetadata(block_metadata));
        let mut state_root = HashValue::zero();
        let mut transaction_hash = vec![];
        for txn in txns {
            let txn_hash = txn.crypto_hash();
            let output = E::execute_transaction(&self.config.vm, chain_state, txn)?;
            match output.status() {
                TransactionStatus::Discard(status) => return Err(status.clone().into()),
                TransactionStatus::Keep(_status) => {
                    //continue.
                }
            }
            state_root = chain_state.commit()?;
            let _transaction_info = TransactionInfo::new(
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
        if let Err(e) = self.verify_proof(accumulator_root, &transaction_hash, first_leaf_idx) {
            warn!("err : {:?}", e);
        }
        if let Err(e) = chain_state.flush() {
            warn!("err : {:?}", e);
        }
        let total_difficulty = {
            let pre_total_difficulty = self
                .get_block_info(block.header().parent_hash())
                .total_difficulty;
            pre_total_difficulty + header.difficult().into()
        };

        let block_info = BlockInfo::new(
            header.id(),
            accumulator_root,
            self.accumulator.get_frozen_subtree_roots().unwrap(),
            self.accumulator.num_leaves(),
            self.accumulator.num_nodes(),
            total_difficulty,
        );
        if let Err(e) = self.commit(block, block_info) {
            warn!("err : {:?}", e);
        }
        Ok(())
    }

    fn commit(&mut self, block: Block, block_info: BlockInfo) -> Result<()> {
        let block_id = block.header().id();
        self.save_block(&block);
        self.chain_info.update_head(block.header().clone());
        self.head = block;
        self.save_block_info(block_info);
        debug!("save block {:?} succ.", block_id);
        Ok(())
    }

    fn chain_state(&mut self) -> &dyn ChainState {
        &self.chain_state
    }
}
