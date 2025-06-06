// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockStorage;
use crate::block_info::legacy::BlockInfoStorage;
use crate::block_info::StcBlockInfoStorage;
use crate::chain_info::ChainInfoStorage;
use crate::contract_event::{legacy::ContractEventStorage, StcContractEventStorage};
use crate::storage::{CodecWriteBatch, ColumnFamily, KeyCodec, SchemaStorage, ValueCodec};
use crate::table_info::{legacy::TableInfoStorage, StcTableInfoStorage};
use crate::transaction::{legacy::TransactionStorage, StcTransactionStorage};
use crate::transaction_info::{
    legacy::{OldTransactionInfoStorage, TransactionInfoStorage},
    StcTransactionInfoStorage,
};
use crate::{
    CodecKVStore, StorageInstance, StorageVersion, BLOCK_BODY_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME,
};
use anyhow::{bail, ensure, format_err, Result};
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{debug, info, warn};
use starcoin_types::block::BlockNumber;
use starcoin_types::startup_info::{BarnardHardFork, DragonHardFork, StartupInfo};
use starcoin_types::transaction::{legacy::RichTransactionInfo, Transaction};
use std::cmp::Ordering;

pub const DEFAULT_UPGRADE_BATCH_SIZE: usize = 1024 * 100; // 100K for default batch size

pub struct DBUpgrade;

pub static BARNARD_HARD_FORK_HEIGHT: BlockNumber = 16080000;
pub static BARNARD_HARD_FORK_HASH: Lazy<HashValue> = Lazy::new(|| {
    HashValue::from_hex_literal(
        "0xf6fc5d0d737e0b9c5768a62a9b4b3bb79b9b1bc47c62fa9fb3b215157dbed9ac",
    )
    .expect("")
});

pub static DRAGON_HARD_FORK_HEIGHT: BlockNumber = 16801958;
pub static DRAGON_HARD_FORK_HASH: Lazy<HashValue> = Lazy::new(|| {
    HashValue::from_hex_literal(
        "0xbef8d0af3b358af9fe25f7383fd2580679c54fe2ce7ff7a7434785ba6d11b943",
    )
    .expect("")
});

impl DBUpgrade {
    pub fn check_upgrade(instance: &mut StorageInstance, batch_size: usize) -> Result<()> {
        let version_in_db = {
            let chain_info_storage = ChainInfoStorage::new(instance.clone());
            chain_info_storage.get_storage_version()?
        };
        // make sure Arc::strong_count(&instance) == 1
        let version_in_code = StorageVersion::current_version();
        match version_in_db.cmp(&version_in_code) {
            Ordering::Less => {
                Self::do_upgrade(version_in_db, version_in_code, instance, batch_size)?;
                let chain_info_storage = ChainInfoStorage::new(instance.clone());
                chain_info_storage.set_storage_version(version_in_code)?;
            }
            Ordering::Equal => {
                debug!(
                "Storage check upgrade, storage version in db same as storage version in code: {:?}",
                version_in_code
            );
            }
            Ordering::Greater => {
                bail!("Storage check upgrade failed, Can not run old starcoin on new storage, storage version in db:{:?}, storage version in code: {:?}", version_in_db, version_in_code);
            }
        }

        Ok(())
    }

    fn db_upgrade_v1_v2(instance: &mut StorageInstance) -> Result<()> {
        let old_transaction_info_storage = OldTransactionInfoStorage::new(instance.clone());
        let block_storage = BlockStorage::new(instance.clone());
        let block_info_storage = BlockInfoStorage::new(instance.clone());
        let transaction_info_storage = TransactionInfoStorage::new(instance.clone());
        let transaction_storage = TransactionStorage::new(instance.clone());
        let mut iter = old_transaction_info_storage.iter()?;
        iter.seek_to_first();
        let mut processed_count = 0;
        for item in iter {
            let (id, old_transaction_info) = item?;
            let block_id = old_transaction_info.block_id;
            let (block, block_info) = match (
                block_storage.get(block_id)?,
                block_info_storage.get(block_id)?,
            ) {
                (Some(block), Some(block_info)) => (block, block_info),
                (_, _) => {
                    warn!("Can not find block or block_info by id: {}, skip this record: {:?}, maybe this transaction info is invalid record. You can use the command `node manager re-execute-block {}` to re execute the block.", block_id, old_transaction_info, block_id);
                    continue;
                }
            };
            let block_number = block.header().number();

            //user transaction start from 1, 0 is block metadata transaction, but the genesis transaction is user transaction, and transaction_index is 0.
            //genesis block s no block metadata transaction.
            //if txn hash not find in block, the txn should be a block metadata transaction.
            let transaction_index = if block_number == 0 {
                0
            } else {
                block
                    .body
                    .transactions
                    .iter()
                    .enumerate()
                    .find_map(|(idx, txn)| {
                        if txn.id() == old_transaction_info.txn_info.transaction_hash {
                            Some(idx + 1)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0) as u32
            };
            //check the transaction.
            if block_number != 0 {
                let transaction = transaction_storage
                    .get(old_transaction_info.txn_info.transaction_hash)?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find transaction by {}",
                            old_transaction_info.txn_info.transaction_hash
                        )
                    })?;
                if transaction_index == 0 {
                    ensure!(
                            matches!(transaction, Transaction::BlockMetadata(_)),
                            "transaction_index 0 must been BlockMetadata transaction, but got txn: {:?}, block:{:?}", transaction, block
                        );
                } else {
                    ensure!(
                            matches!(transaction, Transaction::UserTransaction(_)),
                            "transaction_index > 0 must been UserTransaction transaction, but got txn: {:?}, block:{:?}", transaction, block
                        );
                }
            }
            let txn_len = block.body.transactions.len() + 1;

            let transaction_global_index = if block_number == 0 {
                0
            } else {
                (block_info.txn_accumulator_info.num_leaves - txn_len as u64)
                    + transaction_index as u64
            };
            let rich_transaction_info = RichTransactionInfo::new(
                block_id,
                block_number,
                old_transaction_info.txn_info,
                transaction_index,
                transaction_global_index,
            );
            transaction_info_storage.save_transaction_infos(vec![rich_transaction_info.clone()])?;
            debug!("process transaction_info: {:?}", rich_transaction_info);
            old_transaction_info_storage.remove(id)?;
            processed_count += 1;
            if processed_count % 10000 == 0 {
                info!("processed items: {}", processed_count);
            }
        }
        Ok(())
    }

    fn db_upgrade_v2_v3(instance: &mut StorageInstance) -> Result<()> {
        // https://github.com/facebook/rocksdb/issues/1295
        instance
            .db_mut()
            .unwrap()
            .drop_unused_cfs(vec![BLOCK_BODY_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME])?;
        info!(
            "remove unused column {}, column {}",
            BLOCK_BODY_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME
        );
        Ok(())
    }

    fn db_upgrade_v3_v4(instance: &mut StorageInstance, batch_size: usize) -> Result<()> {
        let old_table = TableInfoStorage::new(instance.clone());
        let new_table = StcTableInfoStorage::new(instance.clone());
        let num = upgrade_store(old_table, new_table, batch_size)?;
        info!("upgrade table storage, total items: {}", num);

        let old_event = ContractEventStorage::new(instance.clone());
        let new_event = StcContractEventStorage::new(instance.clone());
        let num = upgrade_store(old_event, new_event, batch_size)?;
        info!("upgrade contract event storage, total items: {}", num);

        let old_transaction = TransactionStorage::new(instance.clone());
        let new_transaction = StcTransactionStorage::new(instance.clone());
        let num = upgrade_store(old_transaction, new_transaction, batch_size)?;
        info!("upgrade transaction storage, total items: {}", num);

        let old_transaction_info = TransactionInfoStorage::new(instance.clone());
        let new_transaction_info = StcTransactionInfoStorage::new(instance.clone());
        let num = upgrade_store(old_transaction_info, new_transaction_info, batch_size)?;
        info!("upgrade transaction info storage, total items: {}", num);

        let unused_cfs = StorageVersion::V4.cfs_to_be_dropped_since_last_version();
        instance
            .db_mut()
            .unwrap()
            .drop_unused_cfs(unused_cfs.clone())?;
        info!(
            "remove unused columns {:?} for StorageVersion V4",
            unused_cfs
        );

        Ok(())
    }

    pub fn do_upgrade(
        version_in_db: StorageVersion,
        version_in_code: StorageVersion,
        instance: &mut StorageInstance,
        batch_size: usize,
    ) -> Result<()> {
        info!(
            "Upgrade db from {:?} to {:?}",
            version_in_db, version_in_code
        );
        match (version_in_db, version_in_code) {
            (StorageVersion::V1, StorageVersion::V2) => {
                Self::db_upgrade_v1_v2(instance)?;
            }

            (StorageVersion::V1, StorageVersion::V3) => {
                Self::db_upgrade_v1_v2(instance)?;
                Self::db_upgrade_v2_v3(instance)?;
            }

            (StorageVersion::V2, StorageVersion::V3) => {
                Self::db_upgrade_v2_v3(instance)?;
            }

            (StorageVersion::V1, StorageVersion::V4) => {
                Self::db_upgrade_v1_v2(instance)?;
                Self::db_upgrade_v2_v3(instance)?;
                Self::db_upgrade_v3_v4(instance, batch_size)?;
            }
            (StorageVersion::V3, StorageVersion::V4) => {
                Self::db_upgrade_v3_v4(instance, batch_size)?;
            }
            _ => bail!(
                "Can not upgrade db from {:?} to {:?}",
                version_in_db,
                version_in_code
            ),
        }
        Ok(())
    }

    pub fn barnard_hard_fork(instance: &mut StorageInstance) -> Result<()> {
        let block_storage = BlockStorage::new(instance.clone());
        let chain_info_storage = ChainInfoStorage::new(instance.clone());
        let barnard_hard_fork = chain_info_storage.get_barnard_hard_fork()?;

        let barnard_info = BarnardHardFork::new(BARNARD_HARD_FORK_HEIGHT, *BARNARD_HARD_FORK_HASH);
        if barnard_hard_fork == Some(barnard_info.clone()) {
            info!("barnard had forked");
            return Ok(());
        }

        let block = block_storage.get_block_by_hash(*BARNARD_HARD_FORK_HASH)?;
        if let Some(block) = block {
            if block.header().number() == BARNARD_HARD_FORK_HEIGHT {
                info!("barnard hard fork rollback height");
                let mut processed_count = 0;
                let block_info_storage = StcBlockInfoStorage::new(instance.clone());
                let mut iter = block_storage.header_store.iter()?;
                iter.seek_to_first();
                for item in iter {
                    let (id, block_header) = item?;
                    if block_header.number() >= BARNARD_HARD_FORK_HEIGHT {
                        block_info_storage.remove(id)?;
                        processed_count += 1;
                        if processed_count % 10000 == 0 {
                            info!(
                                "barnard hard fork rollback height processed items: {}",
                                processed_count
                            );
                        }
                    }
                }
                let main_hash = block.header().parent_hash();
                chain_info_storage.save_barnard_hard_fork(barnard_info)?;
                chain_info_storage.save_startup_info(StartupInfo::new(main_hash))?;
            }
        }
        Ok(())
    }

    pub fn dragon_hard_fork(instance: &mut StorageInstance) -> Result<()> {
        let block_storage = BlockStorage::new(instance.clone());
        let chain_info_storage = ChainInfoStorage::new(instance.clone());
        let hard_fork = chain_info_storage.get_dragon_hard_fork()?;

        let fork_info = DragonHardFork::new(DRAGON_HARD_FORK_HEIGHT, *DRAGON_HARD_FORK_HASH);
        if hard_fork == Some(fork_info.clone()) {
            info!("dragon hard forked");
            return Ok(());
        }

        let block = block_storage.get_block_by_hash(*DRAGON_HARD_FORK_HASH)?;
        if let Some(block) = block {
            if block.header().number() == DRAGON_HARD_FORK_HEIGHT {
                info!("dragon hard fork rollback height");
                let mut to_deleted = vec![];
                let mut iter = block_storage.header_store.iter()?;
                iter.seek_to_first();
                for item in iter {
                    let (id, block_header) = item?;
                    if block_header.number() > DRAGON_HARD_FORK_HEIGHT {
                        to_deleted.push(id);
                    }
                }
                let block_info_storage = StcBlockInfoStorage::new(instance.clone());
                let mut processed_count = 0;
                for id in to_deleted {
                    block_info_storage.remove(id)?;
                    block_storage.delete_block(id)?;
                    processed_count += 1;
                    if processed_count % 10000 == 0 {
                        info!(
                            "dragon hard fork rollback height processed items: {}",
                            processed_count
                        );
                    }
                }
                if processed_count % 10000 != 0 {
                    info!(
                        "dragon hard fork rollback height processed items: {}",
                        processed_count
                    );
                }
                chain_info_storage.save_dragon_hard_fork(fork_info)?;
                chain_info_storage.save_startup_info(StartupInfo::new(*DRAGON_HARD_FORK_HASH))?;
            }
        }
        Ok(())
    }
}

fn upgrade_store<K1, V1, K2, V2, T1, T2>(
    old_store: T1,
    store: T2,
    batch_size: usize,
) -> Result<usize>
where
    K1: KeyCodec + Into<K2>,
    K2: KeyCodec,
    V1: ValueCodec + Into<V2>,
    V2: ValueCodec,
    T1: SchemaStorage + ColumnFamily<Key = K1, Value = V1>,
    T2: SchemaStorage + ColumnFamily<Key = K2, Value = V2>,
{
    let mut total_size: usize = 0;
    let mut old_iter = old_store.iter()?;
    old_iter.seek_to_first();

    let mut to_put = Some(CodecWriteBatch::<K2, V2>::new());
    let mut item_count = 0;
    let mut no_more = false;

    loop {
        if let Some(item) = old_iter.next() {
            let (id, old_val) = item?;
            let (new_id, new_val) = (id.into(), old_val.into());
            to_put
                .as_mut()
                .unwrap()
                .put(new_id, new_val)
                .expect("should never fail");
            item_count += 1;
        } else {
            no_more = true;
        }
        if item_count == batch_size || no_more {
            if item_count == 0 {
                debug!("no more items to be processed");
                return Ok(total_size);
            }
            debug!(
                "persisting {} items, processed {} items",
                item_count, total_size
            );
            total_size = total_size
                .checked_add(item_count)
                .ok_or_else(|| format_err!("total size overflow, item_count: {}", item_count))?;
            // save new items
            store
                .write_batch(to_put.take().unwrap())
                .expect("should never fail");

            // no more items, let's wrap up
            if no_more {
                return Ok(total_size);
            }

            // reset for next batch
            item_count = 0;
            to_put = Some(CodecWriteBatch::new());
        }
    }
}
