// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Containing panics while validating bytecode from outside.
//!
//! The node's panic hook exits the process on any panic, which runs before `catch_unwind` can
//! see it. Validating a submitted txn deserializes and verifies whatever bytecode it carries, so
//! a single deserializer or verifier panic on crafted input would stop the node. While
//! [`contain`] runs, the hook asks [`is_containing`] and returns instead of exiting; the panic
//! then unwinds into `contain` and becomes an ordinary error. Each validation builds its own VM,
//! so nothing shared is left half-updated. Block execution is not wrapped: a panic there still
//! stops the node, because its state can no longer be trusted.

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

thread_local! {
    static CONTAINING: Cell<bool> = Cell::new(false);
}

/// Whether the current thread is inside [`contain`].
pub fn is_containing() -> bool {
    CONTAINING.with(Cell::get)
}

/// Runs `f`, turning a panic inside it into `Err` with the panic message.
pub fn contain<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    let outer = CONTAINING.with(|c| c.replace(true));
    let result = catch_unwind(AssertUnwindSafe(f));
    CONTAINING.with(|c| c.set(outer));
    result.map_err(|payload| {
        payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_becomes_error_and_flag_resets() {
        assert!(!is_containing());
        let r: Result<(), String> = contain(|| {
            assert!(is_containing());
            panic!("crafted input");
        });
        assert_eq!(r.unwrap_err(), "crafted input");
        assert!(!is_containing());
        assert_eq!(contain(|| 7).unwrap(), 7);
    }
}
