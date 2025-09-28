// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod faucet;
pub mod web;

#[macro_export]
macro_rules! unwrap_or_handle_error {
    ($e:expr, $r:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => return $r(e),
        }
    };
}
