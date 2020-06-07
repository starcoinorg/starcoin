// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::storage::{CodecStorage, KeyCodec, ValueCodec};
use crate::TRANSACTION_INFO_PREFIX_NAME;
use crate::{define_storage, TransactionInfoStore};
use anyhow::{ensure, Error, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crypto::HashValue;
use scs::SCSCodec;
use starcoin_types::transaction::TransactionInfo;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct TxnInfoIndexKey(pub HashValue, pub u64);

define_storage!(
    TransactionInfoStorage,
    TxnInfoIndexKey,
    TransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME
);

impl KeyCodec for TxnInfoIndexKey {
    fn encode_key(&self) -> Result<Vec<u8>, Error> {
        let mut value = self.0.encode_key()?;
        value.write_u64::<BigEndian>(self.1)?;
        Ok(value)
    }

    fn decode_key(key_bytes: &[u8]) -> Result<Self, Error> {
        ensure!(
            key_bytes.len() > HashValue::LENGTH,
            "invalid key length, must great than {}, actual: {}",
            HashValue::LENGTH,
            key_bytes.len()
        );
        let hash_value = HashValue::from_slice(&key_bytes[0..HashValue::LENGTH])?;
        let mut cursor = Cursor::new(&key_bytes[HashValue::LENGTH..]);
        let idx = cursor.read_u64::<BigEndian>()?;
        Ok(TxnInfoIndexKey(hash_value, idx))
    }
}

impl ValueCodec for TransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TransactionInfoStore for TransactionInfoStorage {
    fn get_transaction_info(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>> {
        self.store.get(TxnInfoIndexKey(block_id, idx))
    }

    fn save_transaction_infos(
        &self,
        block_id: HashValue,
        vec_txn_info: Vec<TransactionInfo>,
    ) -> Result<()> {
        let mut batch = WriteBatch::new();

        for (idx, txn_info) in vec_txn_info.into_iter().enumerate() {
            batch.put((block_id, idx as u64), txn_info)?;
        }

        self.store.write_batch(batch)
    }
}
