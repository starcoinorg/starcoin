// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::{CodecStorage, Repository, ValueCodec};
use crate::TRANSACTION_PREFIX_NAME;
use anyhow::Result;
use crypto::HashValue;
use scs::SCSCodec;
use std::sync::Arc;
use types::transaction::TransactionInfo;

define_storage!(
    TransactionInfoStore,
    HashValue,
    TransactionInfo,
    TRANSACTION_PREFIX_NAME
);

impl ValueCodec for TransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
