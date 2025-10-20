// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    define_storage, storage::ValueCodec, STATE_NODE_PREFIX_NAME, STATE_NODE_PREFIX_NAME_V2,
};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_store_api::StateNode;

define_storage!(StateStorage, HashValue, StateNode, STATE_NODE_PREFIX_NAME);
define_storage!(
    StateStorageV2,
    HashValue,
    StateNode,
    STATE_NODE_PREFIX_NAME_V2
);

impl ValueCodec for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.0.clone())
    }
    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(StateNode(data.to_vec()))
    }
}
