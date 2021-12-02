// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::{
    TRANSACTION_INFO_HASH_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME_V2,
};
use anyhow::{Error, Result};
use bcs_ext::BCSCodec;
use crypto::HashValue;
use serde::{Deserialize, Serialize};
use starcoin_types::transaction::{RichTransactionInfo, TransactionInfo};

// This column family is deprecated
define_storage!(
    OldTransactionInfoStorage,
    HashValue,
    BlockTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME
);

define_storage!(
    TransactionInfoStorage,
    HashValue,
    RichTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME_V2
);

define_storage!(
    TransactionInfoHashStorage,
    HashValue,
    Vec<HashValue>,
    TRANSACTION_INFO_HASH_PREFIX_NAME
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockTransactionInfo {
    pub block_id: HashValue,
    pub txn_info: TransactionInfo,
}

impl ValueCodec for BlockTransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for RichTransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TransactionInfoHashStorage {
    pub(crate) fn get_transaction_info_ids_by_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<HashValue>, Error> {
        if let Some(txn_id_vec) = self.get(txn_hash)? {
            Ok(txn_id_vec)
        } else {
            Ok(vec![])
        }
    }

    pub(crate) fn save_transaction_infos(
        &self,
        vec_txn_info: &[RichTransactionInfo],
    ) -> Result<(), Error> {
        let mut batch = CodecWriteBatch::new();
        for txn_info in vec_txn_info {
            if let Some(mut id_vec) = self.get(txn_info.transaction_hash())? {
                if !id_vec.contains(&txn_info.id()) {
                    id_vec.push(txn_info.id());
                    batch.put(txn_info.transaction_hash(), id_vec)?;
                }
            } else {
                batch.put(txn_info.transaction_hash(), vec![txn_info.id()])?;
            }
        }
        self.write_batch(batch)
    }
}
impl TransactionInfoStorage {
    pub(crate) fn get_transaction_info(
        &self,
        id: HashValue,
    ) -> Result<Option<RichTransactionInfo>, Error> {
        self.get(id)
    }
    pub(crate) fn save_transaction_infos(
        &self,
        vec_txn_info: Vec<RichTransactionInfo>,
    ) -> Result<(), Error> {
        let mut batch = CodecWriteBatch::new();
        for txn_info in vec_txn_info {
            batch.put(txn_info.id(), txn_info)?;
        }
        self.write_batch(batch)
    }
}
