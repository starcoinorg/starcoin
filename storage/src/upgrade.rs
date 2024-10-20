// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{
    BlockHeaderStorage, BlockInnerStorage, BlockStorage, DagSyncBlockStorage, FailedBlockStorage,
    OldBlockHeaderStorage, OldBlockInnerStorage, OldFailedBlockStorage,
};
use crate::block_info::BlockInfoStorage;
use crate::chain_info::ChainInfoStorage;
use crate::storage::{CodecWriteBatch, ColumnFamily, KeyCodec, SchemaStorage, ValueCodec};
use crate::transaction::{LegacyTransactionStorage, TransactionStorage};
use crate::transaction_info::OldTransactionInfoStorage;
use crate::transaction_info::TransactionInfoStorage;
use crate::{
    CodecKVStore, RichTransactionInfo, StorageInstance, StorageVersion, BLOCK_BODY_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME,
};
use anyhow::{bail, ensure, format_err, Result};
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{debug, info, warn};
use starcoin_types::block::BlockNumber;
use starcoin_types::startup_info::{BarnardHardFork, DragonHardFork, StartupInfo};
use starcoin_vm_types::transaction::LegacyTransaction;
use std::cmp::Ordering;

pub struct DBUpgrade;

pub static BARNARD_HARD_FORK_HEIGHT: BlockNumber = 16057000;
pub static BARNARD_HARD_FORK_HASH: Lazy<HashValue> = Lazy::new(|| {
    HashValue::from_hex_literal(
        "0x1dd5987fa3b8bffad60f7a7756e73acd7b6808fed5a174200bf49e9f5de2d073",
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
    pub fn check_upgrade(instance: &mut StorageInstance) -> Result<()> {
        let version_in_db = {
            let chain_info_storage = ChainInfoStorage::new(instance.clone());
            chain_info_storage.get_storage_version()?
        };
        // make sure Arc::strong_count(&instance) == 1
        let version_in_code = StorageVersion::current_version();
        match version_in_db.cmp(&version_in_code) {
            Ordering::Less => {
                Self::do_upgrade(version_in_db, version_in_code, instance)?;
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
        // Use old store here, TransactionStorage is using different column family now
        let transaction_storage = LegacyTransactionStorage::new(instance.clone());
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
                    .get_transaction(old_transaction_info.txn_info.transaction_hash)?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find transaction by {}",
                            old_transaction_info.txn_info.transaction_hash
                        )
                    })?;
                if transaction_index == 0 {
                    ensure!(
                            matches!(transaction, LegacyTransaction::BlockMetadata(_)),
                            "transaction_index 0 must been BlockMetadata transaction, but got txn: {:?}, block:{:?}", transaction, block
                        );
                } else {
                    ensure!(
                            matches!(transaction, LegacyTransaction::UserTransaction(_)),
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

    fn db_upgrade_v3_v4(instance: &mut StorageInstance) -> Result<()> {
        upgrade_block_header(instance.clone())?;
        upgrade_transaction(instance.clone())?;

        Ok(())
    }

    fn db_upgrade_v4_v5(instance: &mut StorageInstance) -> Result<()> {
        let _ = DagSyncBlockStorage::new(instance.clone());
        Ok(())
    }

    pub fn do_upgrade(
        version_in_db: StorageVersion,
        version_in_code: StorageVersion,
        instance: &mut StorageInstance,
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
            (StorageVersion::V1, StorageVersion::V4) => {
                Self::db_upgrade_v1_v2(instance)?;
                Self::db_upgrade_v2_v3(instance)?;
                Self::db_upgrade_v3_v4(instance)?;
            }
            (StorageVersion::V1, StorageVersion::V5) => {
                Self::db_upgrade_v1_v2(instance)?;
                Self::db_upgrade_v2_v3(instance)?;
                Self::db_upgrade_v3_v4(instance)?;
                Self::db_upgrade_v4_v5(instance)?;
            }
            (StorageVersion::V2, StorageVersion::V3) => {
                Self::db_upgrade_v2_v3(instance)?;
            }
            (StorageVersion::V2, StorageVersion::V4) => {
                Self::db_upgrade_v2_v3(instance)?;
                Self::db_upgrade_v3_v4(instance)?;
            }
            (StorageVersion::V2, StorageVersion::V5) => {
                Self::db_upgrade_v2_v3(instance)?;
                Self::db_upgrade_v3_v4(instance)?;
                Self::db_upgrade_v4_v5(instance)?;
            }
            (StorageVersion::V3, StorageVersion::V4) => {
                Self::db_upgrade_v3_v4(instance)?;
            }
            (StorageVersion::V3, StorageVersion::V5) => {
                Self::db_upgrade_v3_v4(instance)?;
                Self::db_upgrade_v4_v5(instance)?;
            }
            (StorageVersion::V4, StorageVersion::V5) => {
                Self::db_upgrade_v4_v5(instance)?;
            }
            _ => bail!(
                "Cannot upgrade db from {:?} to {:?}",
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
                let block_info_storage = BlockInfoStorage::new(instance.clone());
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
                let block_info_storage = BlockInfoStorage::new(instance.clone());
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

fn upgrade_store<K, V1, V2, T1, T2>(old_store: T1, store: T2, batch_size: usize) -> Result<usize>
where
    K: KeyCodec + Copy,
    V1: ValueCodec + Into<V2>,
    V2: ValueCodec,
    T1: SchemaStorage + ColumnFamily<Key = K, Value = V1>,
    T2: SchemaStorage + ColumnFamily<Key = K, Value = V2>,
{
    let mut total_size: usize = 0;
    let mut old_iter = old_store.iter()?;
    old_iter.seek_to_first();

    let mut to_delete = Some(CodecWriteBatch::new());
    let mut to_put = Some(CodecWriteBatch::new());
    let mut item_count = 0;

    for item in old_iter {
        let (id, old_block) = item?;
        let block: V2 = old_block.into();
        to_delete
            .as_mut()
            .unwrap()
            .delete(id)
            .expect("should never fail");
        to_put
            .as_mut()
            .unwrap()
            .put(id, block)
            .expect("should never fail");

        item_count += 1;
        if item_count == batch_size {
            total_size = total_size.saturating_add(item_count);
            item_count = 0;
            old_store
                .write_batch(to_delete.take().unwrap())
                .expect("should never fail");
            store
                .write_batch(to_put.take().unwrap())
                .expect("should never fail");

            to_delete = Some(CodecWriteBatch::new());
            to_put = Some(CodecWriteBatch::new());
        }
    }
    if item_count != 0 {
        total_size = total_size.saturating_add(item_count);
        old_store
            .write_batch(to_delete.take().unwrap())
            .expect("should never fail");
        store
            .write_batch(to_put.take().unwrap())
            .expect("should never fail");
    }

    Ok(total_size)
}

fn upgrade_block_header(instance: StorageInstance) -> Result<()> {
    const BATCH_SIZE: usize = 1000usize;

    let old_header_store = OldBlockHeaderStorage::new(instance.clone());
    let header_store = BlockHeaderStorage::new(instance.clone());
    let total_size = upgrade_store(old_header_store, header_store, BATCH_SIZE)?;
    info!("upgraded {total_size} block headers");

    let old_block_store = OldBlockInnerStorage::new(instance.clone());
    let block_store = BlockInnerStorage::new(instance.clone());
    let total_blocks = upgrade_store(old_block_store, block_store, BATCH_SIZE)?;
    info!("upgraded {total_blocks} blocks");

    let old_failed_block_store = OldFailedBlockStorage::new(instance.clone());
    let failed_block_store = FailedBlockStorage::new(instance);
    let total_failed_blocks =
        upgrade_store(old_failed_block_store, failed_block_store, BATCH_SIZE)?;
    info!("upgraded {total_failed_blocks} failed_blocks");

    Ok(())
}

fn upgrade_transaction(instance: StorageInstance) -> Result<()> {
    const BATCH_SIZE: usize = 1000usize;

    let old_txn_store = LegacyTransactionStorage::new(instance.clone());
    let txn_store = TransactionStorage::new(instance);
    let total_size = upgrade_store(old_txn_store, txn_store, BATCH_SIZE)?;
    info!("upgraded {total_size} Transactions");

    Ok(())
}
