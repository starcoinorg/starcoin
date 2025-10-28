// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::TRANSACTION_INFO_HASH_PREFIX_NAME;
use crate::{define_storage, TRANSACTION_INFO_PREFIX_NAME_V3};
use anyhow::{Error, Result};
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::transaction::StcRichTransactionInfo;
use std::collections::HashSet;

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

// it not only store the txn_hash => rich_txn_info_hash
// it also store the txn_info_hash => rich_txn_info_hash
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
        let mut map = HashMap::new();
        for txn_info in vec_txn_info {
            map.entry(txn_info.transaction_hash())
                .and_modify(|v: &mut Vec<HashValue>| v.push(txn_info.id()))
                .or_insert_with(|| vec![txn_info.id()]);

            // some object caches the txn_info_hash before rich info was created
            // it's necessary to supply this mapping mechanism for them to retrieve the rich info.
            map.entry(txn_info.transaction_info.id())
                .and_modify(|v| v.push(txn_info.id()))
                .or_insert_with(|| vec![txn_info.id()]);
        }

        let mut batch = CodecWriteBatch::new();
        for (k, v) in map {
            if let Some(r) = self.get(k)? {
                let merged: Vec<_> = v
                    .into_iter()
                    .chain(r)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                batch.put(k, merged)?;
            } else {
                batch.put(k, v)?;
            }
        }
        self.write_batch(batch)
    }
}

impl StcTransactionInfoStorage {
    pub(crate) fn get_transaction_info_by_rich_info_id(
        &self,
        rich_info_id: HashValue,
    ) -> Result<Option<StcRichTransactionInfo>, Error> {
        self.get(rich_info_id)
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

    pub(crate) fn get_transaction_infos_by_rich_info_id(
        &self,
        rich_info_ids: Vec<HashValue>,
    ) -> Result<Vec<Option<StcRichTransactionInfo>>, Error> {
        self.multiple_get(rich_info_ids)
    }
}
