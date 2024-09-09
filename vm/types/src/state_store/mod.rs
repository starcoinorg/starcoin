use starcoin_crypto::HashValue;
use crate::transaction::Version;

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
pub mod state_key;
pub mod state_value;
pub mod table;
pub mod state_storage_usage;
pub mod errors;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StateViewId {
    /// State-sync applying a chunk of transactions.
    ChunkExecution { first_version: Version },
    /// LEC applying a block.
    BlockExecution { block_id: HashValue },
    /// VmValidator verifying incoming transaction.
    TransactionValidation { base_version: Version },
    /// For test, db-bootstrapper, etc. Usually not aimed to pass to VM.
    Miscellaneous,
}