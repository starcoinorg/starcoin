// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::ValueCodec;
use crate::TRANSACTION_PREFIX_NAME;
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::transaction::Transaction;

define_storage!(
    TransactionStorage,
    HashValue,
    Transaction,
    TRANSACTION_PREFIX_NAME
);

impl ValueCodec for Transaction {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
