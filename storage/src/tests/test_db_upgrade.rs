// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::block::legacy::{BlockInnerStorage, FailedBlockStorage, OldFailedBlock};
use crate::block::{BlockHeaderStorage, StcBlockInnerStorage, StcFailedBlockStorage};
use crate::block_info::legacy::BlockInfoStorage;
use crate::block_info::StcBlockInfoStorage;
use crate::cache_storage::CacheStorage;
use crate::contract_event::legacy::ContractEventStorage;
use crate::db_storage::DBStorage;
use crate::storage::{
    CodecKVStore, InnerStore, KVStore, KeyCodec, SchemaStorage, StorageInstance, ValueCodec,
};
use crate::table_info::legacy::TableInfoStorage;
use crate::table_info::TableInfoStore;
use crate::transaction::legacy::TransactionStorage;
use crate::transaction_info::legacy::{BlockTransactionInfo, OldTransactionInfoStorage};
use crate::{BlockTransactionInfoStore, ContractEventStore, Storage, TransactionStore};
use anyhow::Result;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_config::RocksdbConfig;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_types::block::{Block, BlockHeaderExtra, BlockNumber};
use starcoin_types::{
    account_address::AccountAddress,
    block::{legacy, BlockHeader},
    language_storage::TypeTag,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    vm_error::KeptVMStatus,
};
use starcoin_uint::U256;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::event::EventKey;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::path::Path;

fn random_header_with_number(number: BlockNumber) -> BlockHeader {
    BlockHeader::new(
        HashValue::random(),
        rand::random(),
        number,
        AccountAddress::random(),
        HashValue::random(),
        HashValue::random(),
        HashValue::random(),
        rand::random(),
        U256::MAX,
        HashValue::random(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
    )
}

fn generate_old_db(
    path: &Path,
) -> Result<(
    Vec<HashValue>,
    Vec<HashValue>,
    Vec<HashValue>,
    Vec<TableHandle>,
    Vec<HashValue>,
    Vec<HashValue>,
)> {
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(path, RocksdbConfig::default(), None)?,
    );
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance.clone());
    let transaction_storage = TransactionStorage::new(instance.clone());
    let block_header_storage = BlockHeaderStorage::new(instance.clone());
    let block_storage = BlockInnerStorage::new(instance.clone());
    let block_info_storage = BlockInfoStorage::new(instance.clone());

    let block_header = random_header_with_number(1);
    let txn = SignedUserTransaction::mock();
    let block = legacy::Block {
        header: block_header.clone(),
        body: legacy::BlockBody {
            transactions: vec![txn.clone()],
            uncles: None,
        },
    };
    let mut txn_inf_ids = vec![];
    let mut txn_ids = vec![];
    let block_metadata = {
        // convert to latest type to construct block-meta txn just for convenient.
        let b = Block::from(block.clone());
        b.to_metadata(0)
    };
    let txn_info_0 = TransactionInfo::new(
        block_metadata.id(),
        HashValue::random(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );

    let txn_meta = Transaction::BlockMetadata(block_metadata);
    txn_ids.push(txn_meta.id());
    transaction_storage.put(txn_meta.id(), txn_meta)?;
    txn_inf_ids.push(txn_info_0.id());
    let txn_info_1 = TransactionInfo::new(
        txn.id(),
        HashValue::random(),
        vec![].as_slice(),
        100,
        KeptVMStatus::Executed,
    );
    txn_inf_ids.push(txn_info_1.id());
    let block_info = legacy::BlockInfo {
        block_id: block_header.id(),
        total_difficulty: 0.into(),
        txn_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 2, 3),
        block_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 1, 1),
    };
    let user_txn = Transaction::UserTransaction(txn);
    txn_ids.push(user_txn.id());
    transaction_storage.put(user_txn.id(), user_txn)?;
    //commit_block(block)?;
    block_header_storage.put(block_header.id(), block_header.clone())?;
    block_storage.put(block_header.id(), block.clone())?;
    //save_block_info(block_info)?;
    block_info_storage.put(block_info.block_id, block_info.clone())?;

    old_transaction_info_storage.put(
        txn_info_0.id(),
        BlockTransactionInfo {
            block_id: block_header.id(),
            txn_info: txn_info_0,
        },
    )?;
    old_transaction_info_storage.put(
        txn_info_1.id(),
        BlockTransactionInfo {
            block_id: block_header.id(),
            txn_info: txn_info_1,
        },
    )?;

    let event_ids = generate_old_db_with_contract_event(instance.clone())?;
    let table_handles = generate_old_db_with_table_info(instance.clone())?;
    let (mut blocks, failed_blocks) = generate_old_block_and_block_info(instance)?;
    blocks.push(block_header.id());

    Ok((
        txn_inf_ids,
        txn_ids,
        event_ids,
        table_handles,
        blocks,
        failed_blocks,
    ))
}

fn generate_old_block_and_block_info(
    instance: StorageInstance,
) -> Result<(Vec<HashValue>, Vec<HashValue>)> {
    let block_header_storage = BlockHeaderStorage::new(instance.clone());
    let block_storage = BlockInnerStorage::new(instance.clone());
    let block_info_storage = BlockInfoStorage::new(instance.clone());
    let failed_block_storage = FailedBlockStorage::new(instance.clone());

    let num_blocks = 5u64;
    let num_failed_blocks = 2u64;
    let mut blocks = Vec::with_capacity(num_blocks as usize);
    let mut failed_blocks = Vec::with_capacity(num_failed_blocks as usize);
    for i in 0..num_blocks {
        let block_header = random_header_with_number(2 + i);
        let block_id = block_header.id();
        let block = legacy::Block {
            header: block_header.clone(),
            body: legacy::BlockBody {
                transactions: vec![],
                uncles: None,
            },
        };
        let block_info = legacy::BlockInfo {
            block_id: block_header.id(),
            total_difficulty: 0.into(),
            txn_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 2, 3),
            block_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 1, 1),
        };

        blocks.push(block_id);
        // commit_block
        block_header_storage.put(block_id, block_header)?;
        block_storage.put(block_id, block)?;
        // save block_info
        block_info_storage.put(block_id, block_info)?;
    }

    for i in 0..num_failed_blocks {
        let block_header = random_header_with_number(num_blocks + 2 + i);
        let block_id = block_header.id();
        let block = legacy::Block {
            header: block_header.clone(),
            body: legacy::BlockBody {
                transactions: vec![],
                uncles: None,
            },
        };
        if i == 0 {
            let old_failed_block = OldFailedBlock {
                block,
                peer_id: None,
                failed: format!("test old failed block {block_id}"),
            };
            failed_block_storage
                .get_store()
                .put(block_id.encode_key()?, old_failed_block.encode_value()?)?;
        } else {
            let failed_block = crate::block::legacy::FailedBlock {
                block,
                peer_id: None,
                failed: format!("test failed block{block_id}"),
                version: "v1".to_string(), // if not empty, it will be treated as a new version
            };
            failed_block_storage.put(block_id, failed_block)?;
        };
        info!("insert failed block: {}, idx {}", block_id, i);
        failed_blocks.push(block_id);
    }

    Ok((blocks, failed_blocks))
}

fn generate_old_db_with_contract_event(instance: StorageInstance) -> Result<Vec<HashValue>> {
    let contract_event_storage = ContractEventStorage::new(instance.clone());
    let mut contract_event_ids = vec![];

    for _ in 0..10 {
        let events = (0..10)
            .map(|i| ContractEvent::new(EventKey::random(), i as u64, TypeTag::Bool, vec![0u8; 32]))
            .collect::<Vec<_>>();
        // just use the event hash as the id for simplicity
        let key = events[0].crypto_hash();
        contract_event_storage.put(key, events)?;
        contract_event_ids.push(key);
    }
    Ok(contract_event_ids)
}

fn generate_old_db_with_table_info(instance: StorageInstance) -> Result<Vec<TableHandle>> {
    let table_info_storage = TableInfoStorage::new(instance.clone());
    let mut table_handles = vec![];

    for i in 0..12u8 {
        let table_handle = TableHandle(AccountAddress::new([i; AccountAddress::LENGTH]));
        let table_info = TableInfo::new(TypeTag::U8, TypeTag::U8);
        table_info_storage.put(table_handle, table_info)?;
        table_handles.push(table_handle);
    }
    Ok(table_handles)
}

#[stest::test]
fn test_db_upgrade() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let (txn_info_ids, txn_ids, event_ids, table_handles, blocks, failed_blocks) =
        generate_old_db(tmpdir.path())?;
    let mut instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );

    // set batch size to 5 to trigger multi-batches upgrade for ContractEventStorage and TableInfoStorage
    instance.check_upgrade(5)?;
    let storage = Storage::new(instance.clone())?;
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance.clone());

    for txn_info_id in txn_info_ids {
        assert!(
            old_transaction_info_storage.get(txn_info_id)?.is_none(),
            "expect BlockTransactionInfo is none"
        );
        assert!(
            storage.get_transaction_info(txn_info_id)?.is_some(),
            "expect RichTransactionInfo is some"
        );
    }
    let transaction_storage = TransactionStorage::new(instance.clone());
    for txn_id in txn_ids {
        assert!(
            transaction_storage.get(txn_id)?.is_none(),
            "expect Transaction is some"
        );
        assert!(
            storage
                .get_transaction(txn_id)?
                .and_then(|txn| txn.to_v1())
                .is_some(),
            "expect Transaction is some"
        );
    }

    let contract_event_storage = ContractEventStorage::new(instance.clone());
    for event_id in event_ids {
        assert!(
            contract_event_storage.get(event_id)?.is_none(),
            "expect ContractEvent is none"
        );

        assert_eq!(
            storage
                .get_contract_events_v2(event_id)?
                .map(|e| e.into_iter().all(|e| e.to_v1().is_some())),
            Some(true),
            "expect ContractEvents is none"
        );
    }

    let table_info_storage = TableInfoStorage::new(instance.clone());
    for table_handle in table_handles {
        assert!(
            table_info_storage.get(table_handle)?.is_none(),
            "expect TableInfo is none"
        );
        assert!(
            storage
                .get_table_info(table_handle.into())?
                .and_then(|ti| ti.to_v1())
                .is_some(),
            "expect TableInfo is none"
        );
    }

    let block_storage = BlockInnerStorage::new(instance.clone());
    let new_block_storage = StcBlockInnerStorage::new(instance.clone());
    let block_info_storage = BlockInfoStorage::new(instance.clone());
    let new_block_info_storage = StcBlockInfoStorage::new(instance.clone());

    for block_id in blocks {
        assert!(
            block_storage.get(block_id)?.is_none(),
            "expect Block is none"
        );
        assert!(
            new_block_storage.get(block_id)?.is_some(),
            "expect Block is some"
        );
        assert!(
            block_info_storage.get(block_id)?.is_none(),
            "expect BlockInfo is none"
        );
        assert!(
            new_block_info_storage.get(block_id)?.is_some(),
            "expect BlockInfo is some"
        );
    }

    let failed_block_storage = FailedBlockStorage::new(instance.clone());
    let new_failed_block_storage = StcFailedBlockStorage::new(instance.clone());

    for (idx, failed_block_id) in failed_blocks.into_iter().enumerate() {
        assert!(
            failed_block_storage.get(failed_block_id)?.is_none(),
            "expect FailedBlock is none"
        );
        let Some(failed_block) = new_failed_block_storage.get(failed_block_id)? else {
            panic!("expect FailedBlock is some");
        };
        let (_block, _peer_id, _failed, version) = failed_block.into();
        if idx == 0 {
            assert!(
                version.is_empty(),
                "expect old failed block version is empty"
            );
        } else {
            assert_eq!(
                version,
                "v1".to_string(),
                "expect new failed block version is v1"
            );
        }
    }

    Ok(())
}
