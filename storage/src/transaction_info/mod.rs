// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod legacy;

use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::TRANSACTION_INFO_HASH_PREFIX_NAME;
use crate::{define_storage, TRANSACTION_INFO_PREFIX_NAME_V3};
use anyhow::{Error, Result};
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::transaction::StcRichTransactionInfo;

define_storage!(
    StcTransactionInfoStorage,
    HashValue,
    StcRichTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME_V3
);

define_storage!(
    TransactionInfoHashStorage,
    HashValue,
    Vec<HashValue>,
    TRANSACTION_INFO_HASH_PREFIX_NAME
);

impl ValueCodec for StcRichTransactionInfo {
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
        vec_txn_info: &[StcRichTransactionInfo],
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
impl StcTransactionInfoStorage {
    pub(crate) fn get_transaction_info(
        &self,
        id: HashValue,
    ) -> Result<Option<StcRichTransactionInfo>, Error> {
        self.get(id)
    }
    pub(crate) fn save_transaction_infos(
        &self,
        vec_txn_info: Vec<StcRichTransactionInfo>,
    ) -> Result<(), Error> {
        let mut batch = CodecWriteBatch::new();
        for txn_info in vec_txn_info {
            batch.put(txn_info.id(), txn_info)?;
        }
        self.write_batch(batch)
    }

    pub(crate) fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<StcRichTransactionInfo>>, Error> {
        self.multiple_get(ids)
    }
}
