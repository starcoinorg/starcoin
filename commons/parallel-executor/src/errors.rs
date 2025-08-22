// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
pub enum Error<E> {
    /// Invariant violation that happens internally inside of scheduler, usually an indication of
    /// implementation error.
    InvariantViolation,
    /// Execution of a thread yields a non-recoverable error, such error will be propagated back to
    /// the caller.
    UserError(E),

    BlockRestart,
}

pub type Result<T, E> = ::std::result::Result<T, Error<E>>;
