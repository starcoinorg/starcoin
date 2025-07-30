// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![deny(clippy::arithmetic_side_effects)]
mod chain;
pub mod verifier;
pub use chain::BlockChain;
pub use starcoin_chain_api::{ChainReader, ChainWriter};
pub mod chain_common_func;
