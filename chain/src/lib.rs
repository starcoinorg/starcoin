// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::arithmetic_side_effects)]
mod chain;
mod fixed_blocks;
pub mod verifier;

pub use chain::BlockChain;
pub use starcoin_chain_api::{ChainReader, ChainWriter};
