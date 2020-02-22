// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_store::BlockStore;
use crate::storage::Repository;
use crate::transaction_info_store::TransactionInfoStore;
use anyhow::Result;
use std::sync::Arc;

pub mod block_store;
pub mod memory_storage;
pub mod persistence_storage;
pub mod storage;
pub mod transaction_info_store;

pub struct StarcoinStorage {
    transaction_info_store: TransactionInfoStore,
    pub block_store: BlockStore,
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
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate chrono;
    use crate::memory_storage::MemoryStorage;
    use anyhow::Result;
    use chrono::prelude::*;
    use crypto::{hash::CryptoHash, HashValue};
    use std::time::SystemTime;
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
        let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()]);
        let block1 = Block::new(block_header1.clone(), block_body1);
        let block_id = block1.header().id();
        // save block1
        storage.block_store.save(block1.clone());
        //read to block2
        let block2 = storage.block_store.get(block_id).unwrap();
        assert!(block2.is_some());
        assert_eq!(block1, block2.unwrap());
        //get header to block3
        let block_header3 = storage
            .block_store
            .get_block_header_by_hash(block_id)
            .unwrap();
        assert_eq!(block_header1, block_header3);
    }

    #[test]
    fn test_block_number() {
        let store = Arc::new(MemoryStorage::new());
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
            .unwrap();
        assert_eq!(block1, block3);
        //get header by number
        // let block4_header = storage
        //     .block_store
        //     .get_block_header_by_number(block_number1)
        //     .unwrap();
        // assert_eq!(block_header1, block4_header);
        // get latest block
        let block5 = storage.block_store.get_latest_block().unwrap();
        assert_eq!(block1, block5);
    }
}
