// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator::AccumulatorStore;
use crate::block::BlockStore;
use crate::block_info::{BlockInfoStorage, BlockInfoStore};
use crate::state_node::StateNodeStorage;
use crate::storage::Repository;
use crate::transaction_info::TransactionInfoStore;
use anyhow::{ensure, Error, Result};
use crypto::HashValue;
use starcoin_accumulator::{
    node_index::NodeIndex, AccumulatorNode, AccumulatorNodeReader, AccumulatorNodeStore,
    AccumulatorNodeWriter,
};
use state_tree::{StateNode, StateNodeStore};
use std::convert::TryInto;
use std::sync::Arc;
use types::{
    block::{Block, BlockBody, BlockHeader, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};

pub mod accumulator;
pub mod block;
pub mod block_info;
pub mod memory_storage;
pub mod persistence_storage;
pub mod state_node;
pub mod storage;
pub mod transaction_info;

pub type KeyPrefixName = &'static str;

pub trait BlockStorageOp {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn save(&self, block: Block) -> Result<()>;

    fn save_header(&self, header: BlockHeader) -> Result<()>;

    fn get_headers(&self) -> Result<Vec<HashValue>>;

    fn save_body(&self, block_id: HashValue, body: BlockBody) -> Result<()>;

    fn save_number(&self, number: BlockNumber, block_id: HashValue) -> Result<()>;

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>>;

    fn get_number(&self, number: u64) -> Result<Option<HashValue>>;

    fn commit_block(&self, block: Block) -> Result<()>;

    fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>>;

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>>;

    fn get_latest_block(&self) -> Result<Block>;

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>>;

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>>;

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>>;

    fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>>;
}

pub struct StarcoinStorage {
    transaction_info_store: TransactionInfoStore,
    pub block_store: BlockStore,
    state_node_store: StateNodeStorage,
    accumulator_store: AccumulatorStore,
    block_info_store: BlockInfoStore,
    //TODO implement storage.
    startup_info_store: Arc<dyn Repository>,
}

impl StarcoinStorage {
    pub fn new(storage: Arc<dyn Repository>) -> Result<Self> {
        Ok(Self {
            transaction_info_store: TransactionInfoStore::new(storage.clone()),
            block_store: BlockStore::new(
                storage.clone(),
                storage.clone(),
                storage.clone(),
                storage.clone(),
                storage.clone(),
            ),
            state_node_store: StateNodeStorage::new(storage.clone()),
            accumulator_store: AccumulatorStore::new(storage.clone()),
            block_info_store: BlockInfoStore::new(storage.clone()),
            startup_info_store: storage.clone(),
        })
    }
}

impl StateNodeStore for StarcoinStorage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        self.state_node_store.get(hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.state_node_store.put(key, node)
    }
}

impl BlockStorageOp for StarcoinStorage {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.startup_info_store
            .get("startup_info".as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.startup_info_store
            .put("starup_info".as_bytes().to_vec(), startup_info.try_into()?)
    }

    fn save(&self, block: Block) -> Result<()> {
        self.block_store.save(block)
    }

    fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.block_store.save_header(header)
    }

    fn get_headers(&self) -> Result<Vec<HashValue>> {
        self.block_store.get_headers()
    }

    fn save_body(&self, block_id: HashValue, body: BlockBody) -> Result<()> {
        self.block_store.save_body(block_id, body)
    }

    fn save_number(&self, number: BlockNumber, block_id: HashValue) -> Result<()> {
        self.block_store.save_number(number, block_id)
    }

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_store.get(block_id)
    }

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.block_store.get_body(block_id)
    }

    fn get_number(&self, number: u64) -> Result<Option<HashValue>> {
        self.block_store.get_number(number)
    }

    fn commit_block(&self, block: Block) -> Result<()> {
        self.block_store.commit_block(block)
    }

    fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        self.block_store.get_branch_hashes(block_id)
    }

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>> {
        self.block_store.get_latest_block_header()
    }

    fn get_latest_block(&self) -> Result<Block> {
        self.block_store.get_latest_block()
    }

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.block_store.get_block_header_by_hash(block_id)
    }

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_store.get_block_by_hash(block_id)
    }

    fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        self.block_store.get_block_header_by_number(number)
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.block_store.get_block_by_number(number)
    }
    fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        self.block_store.get_common_ancestor(block_id1, block_id2)
    }
}

impl AccumulatorNodeStore for StarcoinStorage {}
impl AccumulatorNodeReader for StarcoinStorage {
    ///get node by node_index
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>> {
        self.accumulator_store.get(index)
    }
    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        self.accumulator_store.get_node(hash)
    }
}

impl AccumulatorNodeWriter for StarcoinStorage {
    /// save node index
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<()> {
        self.accumulator_store.save(index, hash)
    }
    /// save node
    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.accumulator_store.save_node(node)
    }
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()> {
        self.accumulator_store.delete_nodes(node_hash_vec)
    }
    ///delete larger index than one
    fn delete_nodes_index(&self, vec_index: Vec<NodeIndex>) -> Result<()> {
        self.accumulator_store.delete_nodes_index(vec_index)
    }
}

impl BlockInfoStorage for StarcoinStorage {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<(), Error> {
        self.block_info_store.save(block_info)
    }

    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>, Error> {
        self.block_info_store.get(hash_value)
    }
}

//TODO should move this traits to traits crate?
/// Chain storage define
pub trait BlockChainStore:
    StateNodeStore + BlockStorageOp + AccumulatorNodeStore + BlockInfoStorage
{
}

impl BlockChainStore for StarcoinStorage {}

///ensure slice length
fn ensure_slice_len_eq(data: &[u8], len: usize) -> Result<()> {
    ensure!(
        data.len() == len,
        "Unexpected data len {}, expected {}.",
        data.len(),
        len,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate chrono;

    use crate::memory_storage::MemoryStorage;

    use chrono::prelude::*;
    use crypto::{hash::CryptoHash, HashValue};

    use types::account_address::AccountAddress;
    use types::block::{Block, BlockBody, BlockHeader};
    use types::transaction::{SignedUserTransaction, TransactionInfo};
    use types::vm_error::StatusCode;

    #[test]
    fn test_storage() {
        let store = Arc::new(MemoryStorage::new());
        let storage = StarcoinStorage::new(store).unwrap();
        let transaction_info1 = TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            StatusCode::ABORTED,
        );
        let id = transaction_info1.crypto_hash();
        storage
            .transaction_info_store
            .save(transaction_info1.clone());
        let transaction_info2 = storage.transaction_info_store.get(id).unwrap();
        assert!(transaction_info2.is_some());
        assert_eq!(transaction_info1, transaction_info2.unwrap());
    }

    #[test]
    fn test_block() {
        let store = Arc::new(MemoryStorage::new());
        // let block_store = Arc::new(MemoryStorage::new());
        // let header_store = Arc::new(MemoryStorage::new());
        // let sons_store = Arc::new(MemoryStorage::new());
        // let body_store = Arc::new(MemoryStorage::new());
        let storage = StarcoinStorage::new(store).unwrap();
        let consensus_header = vec![0u8; 1];
        let dt = Local::now();

        let block_header1 = BlockHeader::new(
            HashValue::random(),
            dt.timestamp_nanos() as u64,
            1,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header,
        );
        storage.block_store.save_header(block_header1.clone());
        let block_id = block_header1.id();
        assert_eq!(
            block_header1,
            storage
                .block_store
                .get_block_header_by_hash(block_id.clone())
                .unwrap()
                .unwrap()
        );
        let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()]);
        storage
            .block_store
            .save_body(block_id.clone(), block_body1.clone());
        let block1 = Block::new(block_header1.clone(), block_body1);
        // save block1
        storage.block_store.save(block1.clone());
        //read to block2
        let block2 = storage.block_store.get(block_id.clone()).unwrap();
        assert!(block2.is_some());
        assert_eq!(block1, block2.unwrap());
        //get header to block3
        let block_header3 = storage
            .block_store
            .get_block_header_by_hash(block_id)
            .unwrap()
            .unwrap();
        assert_eq!(block_header1, block_header3);
    }

    #[test]
    fn test_block_number() {
        let store = Arc::new(MemoryStorage::new());
        // let block_store = Arc::new(MemoryStorage::new());
        // let header_store = Arc::new(MemoryStorage::new());
        // let sons_store = Arc::new(MemoryStorage::new());
        // let body_store = Arc::new(MemoryStorage::new());
        let storage = StarcoinStorage::new(store).unwrap();
        let consensus_header = vec![0u8; 1];
        let dt = Local::now();

        let block_header1 = BlockHeader::new(
            HashValue::random(),
            dt.timestamp_nanos() as u64,
            0,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header,
        );
        storage.block_store.save_header(block_header1.clone());
        let block_id = block_header1.id();
        assert_eq!(
            storage
                .block_store
                .get_block_header_by_hash(block_id)
                .unwrap()
                .unwrap(),
            block_header1
        );
        let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()]);
        storage.block_store.save_body(block_id, block_body1.clone());
        let block1 = Block::new(block_header1.clone(), block_body1.clone());

        // save block1
        storage.block_store.save(block1.clone());
        let block_number1 = block_header1.number();
        storage.block_store.save_number(block_number1, block_id);
        //read to block2
        let block2 = storage.block_store.get(block_id).unwrap();
        assert!(block2.is_some());
        assert_eq!(block1, block2.unwrap());
        //get number to block3
        let block3 = storage
            .block_store
            .get_block_by_number(block_number1)
            .unwrap()
            .unwrap();
        assert_eq!(block1, block3);
        //get header by number
        let block4_header = storage
            .block_store
            .get_block_header_by_number(block_number1)
            .unwrap()
            .unwrap();
        assert_eq!(block_header1, block4_header);
        //TODO fixme
        // get latest block
        // let block5 = storage.block_store.get_latest_block().unwrap();
        // assert_eq!(block1, block5);
    }

    #[test]
    fn test_block_branch_hashes() {
        let store = Arc::new(MemoryStorage::new());
        let _block_store = Arc::new(MemoryStorage::new());
        let _header_store = Arc::new(MemoryStorage::new());
        let _sons_store = Arc::new(MemoryStorage::new());
        let _body_store = Arc::new(MemoryStorage::new());
        let storage = StarcoinStorage::new(store).unwrap();
        let consensus_header = vec![0u8; 1];
        let dt = Local::now();

        let block_header0 = BlockHeader::new(
            HashValue::random(),
            dt.timestamp_nanos() as u64,
            0,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header.clone(),
        );
        storage.block_store.save_header(block_header0.clone());

        let parent_hash = block_header0.clone().id();
        let block_header1 = BlockHeader::new(
            parent_hash,
            dt.timestamp_nanos() as u64,
            1,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header.clone(),
        );
        storage.block_store.save_header(block_header1.clone());
        let block_id = block_header1.id();
        println!("header1: {}", block_id.to_hex());
        let block_header2 = BlockHeader::new(
            parent_hash,
            dt.timestamp_nanos() as u64,
            2,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header.clone(),
        );
        storage.block_store.save_header(block_header2.clone());
        println!("header2: {}", block_header2.clone().id().to_hex());

        let block_header3 = BlockHeader::new(
            block_id,
            dt.timestamp_nanos() as u64,
            3,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header.clone(),
        );
        storage.block_store.save_header(block_header3.clone());
        println!("header3: {}", block_header3.clone().id().to_hex());

        let block_header4 = BlockHeader::new(
            block_header3.id(),
            dt.timestamp_nanos() as u64,
            4,
            AccountAddress::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            consensus_header,
        );
        storage.block_store.save_header(block_header4.clone());
        println!("header4: {}", block_header4.clone().id().to_hex());
        let hashes = storage
            .block_store
            .get_branch_hashes(block_header4.id())
            .unwrap();
        let desert_vec = vec![block_header3.clone().id(), block_id];
        assert_eq!(hashes, desert_vec);
        let comm_hash = storage
            .block_store
            .get_common_ancestor(block_header1.id(), block_header2.id())
            .unwrap()
            .unwrap();
        assert_eq!(comm_hash, parent_hash);
    }
}
