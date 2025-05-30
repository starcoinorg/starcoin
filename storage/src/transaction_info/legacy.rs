// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::{TRANSACTION_INFO_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME_V2};
use anyhow::{Error, Result};
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::transaction::{legacy::RichTransactionInfo, TransactionInfo};

// This column family is deprecated
define_storage!(
    OldTransactionInfoStorage,
    HashValue,
    BlockTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME
);

// This column family is deprecated
define_storage!(
    TransactionInfoStorage,
    HashValue,
    RichTransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME_V2
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

impl TransactionInfoStorage {
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
