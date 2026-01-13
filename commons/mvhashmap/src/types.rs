// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_aggregator::types::{DelayedFieldsSpeculativeError, PanicOr};
use std::sync::atomic::AtomicUsize;

pub type AtomicTxnIndex = AtomicUsize;
pub type TxnIndex = usize;
pub type Incarnation = usize;

#[derive(Debug, PartialEq, Eq)]
pub enum MVDelayedFieldsError {
    NotFound,
    Dependency(TxnIndex),
    DeltaApplicationFailure,
}

impl MVDelayedFieldsError {
    pub fn from_panic_or(
        err: PanicOr<DelayedFieldsSpeculativeError>,
    ) -> PanicOr<MVDelayedFieldsError> {
        match err {
            PanicOr::CodeInvariantError(e) => PanicOr::CodeInvariantError(e),
            PanicOr::Or(DelayedFieldsSpeculativeError::NotFound(_)) => {
                PanicOr::Or(MVDelayedFieldsError::NotFound)
            }
            PanicOr::Or(_) => PanicOr::Or(MVDelayedFieldsError::DeltaApplicationFailure),
        }
    }
}
