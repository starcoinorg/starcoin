// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.

pub use stest_macro::test;

pub fn init_test_logger() {
    let env = env_logger::Env::new().filter_or("RUST_LOG", "debug");
    let _ = env_logger::from_env(env).is_test(true).try_init();
}
