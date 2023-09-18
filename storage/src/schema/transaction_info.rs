// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    db::{
        TRANSACTION_INFO_HASH_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME_V2,
    },
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_types::transaction::{RichTransactionInfo, TransactionInfo as TxnInfo};

define_schema!(
    TransactionInfo,
    HashValue,
    RichTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME_V2
);

impl KeyCodec<TransactionInfo> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }
    fn decode_key(data: &[u8]) -> Result<Self> {
        <Self as BCSCodec>::decode(data)
    }
}

impl ValueCodec<TransactionInfo> for RichTransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }
    fn decode_value(data: &[u8]) -> Result<Self> {
        <Self as BCSCodec>::decode(data)
    }
}

define_schema!(
    TransactionInfoHash,
    HashValue,
    Vec<HashValue>,
    TRANSACTION_INFO_HASH_PREFIX_NAME
);

impl KeyCodec<TransactionInfoHash> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        <Self as BCSCodec>::decode(data)
    }
}

impl ValueCodec<TransactionInfoHash> for Vec<HashValue> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }
    fn decode_value(data: &[u8]) -> Result<Self> {
        <Self as BCSCodec>::decode(data)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockTransactionInfo {
    pub block_id: HashValue,
    pub txn_info: TxnInfo,
}

// This column family is deprecated
define_schema!(
    OldTransactionInfo,
    HashValue,
    BlockTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME
);

impl KeyCodec<OldTransactionInfo> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        <Self as BCSCodec>::decode(data)
    }
}

impl ValueCodec<OldTransactionInfo> for BlockTransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        <Self as BCSCodec>::decode(data)
    }
}

/*
use crate::storage::CodecWriteBatch;
use anyhow::{Error, Result};
use starcoin_crypto::HashValue;
use starcoin_types::transaction::RichTransactionInfo;

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

    pub(crate) fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<RichTransactionInfo>>, Error> {
        self.multiple_get(ids)
    }
}
*/
