// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::ValueCodec;
use crate::STATE_NODE_PREFIX_NAME;
use anyhow::Result;
use crypto::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use starcoin_state_store_api::StateNode;

define_storage!(StateStorage, HashValue, StateNode, STATE_NODE_PREFIX_NAME);

impl ValueCodec for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.0.encode()
    }
    fn decode_value(data: &[u8]) -> Result<Self> {
        Node::decode(data).map(StateNode)
    }
}
