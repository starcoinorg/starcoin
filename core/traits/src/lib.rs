// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
mod chain_service;
mod consensus;

pub use chain::{Chain, ChainReader, ChainWriter, ExcludedTxns};
pub use chain_service::{ChainAsyncService, ChainService};
pub use consensus::Consensus;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ConnectBlockResult {
    DuplicateConn,
    FutureBlock,
    VerifyBlockIdFailed,
    VerifyConsensusFailed,
    VerifyBodyFailed,
    UncleBlockIllegal,
    DuplicateUncles,
    VerifyTxnInfoFailed,
    SUCCESS,
}
