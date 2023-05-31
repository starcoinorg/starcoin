// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecKVStore, ValueCodec};
use crate::{define_storage, WriteSetStore, WRITE_SET_PRIFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::write_set::WriteSet;

define_storage!(WriteSetStorage, HashValue, WriteSet, WRITE_SET_PRIFIX_NAME);

impl ValueCodec for WriteSet {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl WriteSetStore for WriteSetStorage {
    fn get_write_set(&self, hash: HashValue) -> Result<Option<WriteSet>> {
        self.get(hash)
    }

    fn save_write_set(&self, hash: HashValue, write_set_vec: WriteSet) -> Result<()> {
        self.put(hash, write_set_vec)
    }
}
