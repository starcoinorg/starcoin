// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    /// Invariant violation that happens internally inside of scheduler, usually an indication of
    /// implementation error.
    InvariantViolation,
    /// The same module access path for module was both read & written during speculative executions.
    /// This may trigger a race due to the Move-VM loader cache implementation, and mitigation requires
    /// aborting the parallel execution pipeline and falling back to the sequential execution.
    /// TODO: (short-med term) relax the limitation, and (mid-long term) provide proper multi-versioning
    /// for code (like data) for the cache.
    ModulePathReadWrite,
    /// This may trigger the parallel execution when txn output in Block trigger UpgradeEvent or ConfigChangeEvent
    BlockRestart,
    /// Execution of a thread yields a non-recoverable error, such error will be propagated back to
    /// the caller.
    UserError(E),
}

pub type Result<T, E> = ::std::result::Result<T, Error<E>>;
