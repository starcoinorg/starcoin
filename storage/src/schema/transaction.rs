// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    db::TRANSACTION_PREFIX_NAME,
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_types::transaction::Transaction as TxnType;

define_schema!(Transaction, HashValue, TxnType, TRANSACTION_PREFIX_NAME);

impl KeyCodec<Transaction> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }
    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<Transaction> for TxnType {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
