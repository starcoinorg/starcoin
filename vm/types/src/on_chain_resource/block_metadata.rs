// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::event::EventHandle;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

/// On chain resource BlockMetadata mapping for FlexiDag block
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    // number of the current block
    pub number: u64,
    // Hash of the parent block.
    pub parent_hash: HashValue,
    // Author of the current block.
    pub author: AccountAddress,
    pub uncles: u64,
    // Handle where events with the time of new blocks are emitted
    pub new_block_events: EventHandle,
    // An Array of the parents hash for a Dag block.
    pub parents_hash: Vec<u8>,
}

impl BlockMetadata {
    pub fn parents_hash(&self) -> anyhow::Result<Vec<HashValue>> {
        bcs_ext::from_bytes(self.parents_hash.as_slice())
    }
}

impl MoveStructType for BlockMetadata {
    const MODULE_NAME: &'static IdentStr = ident_str!("stc_block");
    const STRUCT_NAME: &'static IdentStr = ident_str!("BlockMetadata");
}

impl MoveResource for BlockMetadata {}

#[cfg(test)]
mod  tests {
    use starcoin_crypto::HashValue;

    #[test]
    fn test_hash_value_serialize_and_deserialize() {
        let hash = HashValue::zero();
        let bytes = bcs::to_bytes(&hash).unwrap();
        const BUF_SIZE:usize = HashValue::LENGTH + 1;
        let mut buf : [u8; BUF_SIZE] = [0; BUF_SIZE];
        buf[0] = HashValue::LENGTH as u8;
        assert_eq!(bytes.as_slice(), buf);
        let hash1 = bcs::from_bytes::<HashValue>(&bytes).unwrap();
        assert_eq!(hash1,hash);
    }

}