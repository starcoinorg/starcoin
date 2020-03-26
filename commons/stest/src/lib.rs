// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.

pub use stest_macro::test;

pub fn init_test_logger() {
    starcoin_logger::init_for_test();
}
