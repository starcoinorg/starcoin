// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use starcoin_vm_types::transaction::SignedUserTransaction;

mod chain;
mod errors;
pub mod message;
mod service;

#[derive(Clone, Debug)]
pub struct ExcludedTxns {
    pub discarded_txns: Vec<SignedUserTransaction>,
    pub untouched_txns: Vec<SignedUserTransaction>,
}

pub use chain::{Chain, ChainReader, ChainWriter};
pub use errors::*;
pub use service::{ChainAsyncService, ReadableChainService, WriteableChainService};
