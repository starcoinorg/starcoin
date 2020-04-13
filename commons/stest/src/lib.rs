// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.

use std::sync::mpsc::Sender;
pub use stest_macro::test;

pub fn init_test_logger() {
    starcoin_logger::init_for_test();
}

pub fn timeout<F>(timeout: u64, f: F, tx: Sender<()>) -> ()
where
    F: FnOnce(),
    F: Send + 'static,
{
    std::thread::spawn(f);

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(timeout));
        let _ = tx.send(());
    });

    ()
}
