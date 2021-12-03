// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_info::OldTransactionInfoStorage;
use crate::{CodecKVStore, RichTransactionInfo, Storage, StorageVersion};
use anyhow::{bail, format_err, Result};
use logger::prelude::{debug, info};
use std::cmp::Ordering;

pub struct DBUpgrade;

impl DBUpgrade {
    pub fn check_upgrade(storage: Storage) -> Result<Storage> {
        let version_in_db = storage.chain_info_storage.get_storage_version()?;
        let version_in_code = StorageVersion::current_version();
        match version_in_db.cmp(&version_in_code) {
            Ordering::Less => {
                Self::do_upgrade(version_in_db, version_in_code, &storage)?;
                storage
                    .chain_info_storage
                    .set_storage_version(version_in_code)?;
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

        Ok(storage)
    }

    pub fn do_upgrade(
        version_in_db: StorageVersion,
        version_in_code: StorageVersion,
        storage: &Storage,
    ) -> Result<()> {
        info!(
            "Upgrade db from {:?} to {:?}",
            version_in_db, version_in_code
        );
        match (version_in_db, version_in_code) {
            (StorageVersion::V1, StorageVersion::V2) => {
                let old_transaction_info_storage =
                    OldTransactionInfoStorage::new(storage.instance.clone());
                let mut iter = old_transaction_info_storage.iter()?;
                iter.seek_to_first();
                let mut processed_count = 0;
                for item in iter {
                    let (id, old_transaction_info) = item?;
                    let block_id = old_transaction_info.block_id;
                    let block = storage
                        .block_storage
                        .get(block_id)?
                        .ok_or_else(|| format_err!("Can not find block by id: {}", block_id))?;
                    let block_number = block.header().number();
                    //if txn hash not find in block, the txn should be a block metadata transaction.
                    let transaction_index = block
                        .body
                        .transactions
                        .iter()
                        .enumerate()
                        .find_map(|(idx, txn)| {
                            if txn.id() == old_transaction_info.txn_info.transaction_hash {
                                Some(idx)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0) as u32;
                    let block_info =
                        storage.block_info_storage.get(block_id)?.ok_or_else(|| {
                            format_err!("Can not find block info by id: {}", block_id)
                        })?;
                    let transaction_global_index =
                        block_info.txn_accumulator_info.num_leaves + transaction_index as u64;
                    let rich_transaction_info = RichTransactionInfo::new(
                        block_id,
                        block_number,
                        old_transaction_info.txn_info,
                        transaction_index,
                        transaction_global_index,
                    );
                    storage
                        .transaction_info_storage
                        .save_transaction_infos(vec![rich_transaction_info.clone()])?;
                    debug!("process transaction_info: {:?}", rich_transaction_info);
                    old_transaction_info_storage.remove(id)?;
                    processed_count += 1;
                    if processed_count % 10000 == 0 {
                        info!("processed items: {}", processed_count);
                    }
                }
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
