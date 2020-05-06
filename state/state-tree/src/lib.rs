// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod mock;
mod state_tree;

#[cfg(test)]
mod state_tree_test;

pub use starcoin_state_store_api::{StateNode, StateNodeStore};
pub use state_tree::StateTree;

use starcoin_crypto::HashValue;

/// change the `n`th nibble to `nibble`
pub fn update_nibble(original_key: &HashValue, n: usize, nibble: u8) -> HashValue {
    assert!(nibble < 16);
    let mut key = original_key.to_vec();
    key[n / 2] = if n % 2 == 0 {
        key[n / 2] & 0x0f | nibble << 4
    } else {
        key[n / 2] & 0xf0 | nibble
    };
    HashValue::from_slice(&key).unwrap()
}
