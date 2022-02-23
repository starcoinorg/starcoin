// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockStorage;
use crate::block_info::BlockInfoStorage;
use crate::chain_info::ChainInfoStorage;
use crate::transaction::TransactionStorage;
use crate::transaction_info::OldTransactionInfoStorage;
use crate::transaction_info::TransactionInfoStorage;
use crate::{
    CodecKVStore, RichTransactionInfo, StorageInstance, StorageVersion, TransactionStore,
    BLOCK_BODY_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
};
use anyhow::{bail, ensure, format_err, Result};
use logger::prelude::{debug, info, warn};
use starcoin_types::transaction::Transaction;
use std::cmp::Ordering;

pub struct DBUpgrade;

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
                    .get_transaction(old_transaction_info.txn_info.transaction_hash)?
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

            (StorageVersion::V2, StorageVersion::V3) => {
                Self::db_upgrade_v2_v3(instance)?;
            }
            _ => bail!(
                "Can not upgrade db from {:?} to {:?}",
                version_in_db,
                version_in_code
            ),
        }
        Ok(())
    }
}
