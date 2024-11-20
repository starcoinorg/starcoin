// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::event::EventHandle;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

/// On chain resource BlockMetadata mapping for FlexiDag block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    // number of the current block
    pub number: u64,
    // Hash of the parent block.
    pub parent_hash: Vec<u8>,
    // Author of the current block.
    pub author: AccountAddress,
    // Uncle blocks number
    pub uncles: u64,
    // Parents hash for DAG
    pub parents_hash: Vec<u8>,
    // Handle where events with the time of new blocks are emitted
    pub new_block_events: EventHandle,
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
mod tests {
    use starcoin_crypto::HashValue;

    #[test]
    fn test_hash_value_serialize_and_deserialize() {
        let hash = HashValue::zero();
        let bytes = bcs::to_bytes(&hash).unwrap();
        const BUF_SIZE: usize = HashValue::LENGTH + 1;
        let mut buf = [0; BUF_SIZE];
        buf[0] = HashValue::LENGTH as u8;
        assert_eq!(bytes.as_slice(), buf);
        let hash1 = bcs::from_bytes::<HashValue>(&bytes).unwrap();
        assert_eq!(hash1, hash);
    }

    #[test]
    fn test_hash_value_serialize() {
        let hash = HashValue::random();
        let json_value = serde_json::to_string(&hash).unwrap();
        println!("{}", json_value);
        assert_eq!(json_value, format!("\"{}\"", hash.to_string()));

        let de_hash = serde_json::from_slice::<HashValue>(json_value.as_bytes()).unwrap();
        let de_hash2: HashValue = serde_json::from_str::<HashValue>(&json_value).unwrap();
        assert_eq!(hash, de_hash);
        assert_eq!(hash, de_hash2);
    }
}
