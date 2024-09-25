// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod block_executor;

pub use block_executor::*;

/// The system is not in a state where the operation can be performed (http: 400)
pub const INVALID_STATE: u64 = 0x3;

/// Construct a canonical error code from a category and a reason.
pub fn canonical(category: u64, reason: u64) -> u64 {
    (category << 16) + reason
}

pub fn invalid_state(r: u64) -> u64 {
    canonical(INVALID_STATE, r)
}
