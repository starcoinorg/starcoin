// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![deny(clippy::integer_arithmetic)]
mod chain;
mod chain_metrics;
pub mod verifier;
pub use chain::BlockChain;
