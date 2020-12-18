// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
pub mod verifier;

pub use chain::BlockChain;

#[cfg(test)]
mod tests;
