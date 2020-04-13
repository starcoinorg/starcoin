// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use futures::Future;

use std::sync::mpsc::Sender;
use std::time::Duration;
pub use stest_macro::test;

pub fn init_test_logger() {
    starcoin_logger::init_for_test();
}

pub fn timeout<F>(timeout: u64, f: F, tx: Sender<()>)
where
    F: FnOnce(),
    F: Send + 'static,
{
    std::thread::spawn(f);

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(timeout));
        let _ = tx.send(());
    });
}

pub fn make_channel() -> (UnboundedSender<()>, UnboundedReceiver<()>) {
    unbounded()
}

pub async fn timeout_future(timeout: u64, tx: UnboundedSender<()>) {
    actix::clock::delay_for(Duration::from_secs(timeout)).await;
    let _ = tx.unbounded_send(());
}

pub async fn test_future<F>(f: F, tx: UnboundedSender<()>)
where
    F: Future<Output = ()>,
{
    f.await;
    let _ = tx.unbounded_send(());
}

pub async fn wait_result(mut rx: UnboundedReceiver<()>) {
    let _ = rx.next().await;
}
