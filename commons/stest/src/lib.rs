// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use futures::Future;

use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
pub use stest_macro::test;

pub fn init_test_logger() {
    starcoin_logger::init_for_test();
}

pub fn timeout<F, T>(timeout: u64, f: F, tx: Sender<Option<T>>)
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(timeout));
        let _ = tx.send(None);
    });

    std::thread::spawn(move || {
        let t = f();
        let _ = tx_clone.send(Some(t));
    });
}

pub fn wait_channel<T>(rx: Receiver<Option<T>>) -> T {
    let result = rx.recv();
    match result {
        Ok(Some(t)) => t,
        _ => panic!(),
    }
}

pub fn make_channel<T>() -> (UnboundedSender<Option<T>>, UnboundedReceiver<Option<T>>) {
    unbounded()
}

pub async fn timeout_future<T>(timeout: u64, tx: UnboundedSender<Option<T>>) {
    actix::clock::delay_for(Duration::from_secs(timeout)).await;
    let _ = tx.unbounded_send(None);
}

pub async fn test_future<F, T>(f: F, tx: UnboundedSender<Option<T>>)
where
    F: Future<Output = T>,
{
    let t = f.await;
    let _ = tx.unbounded_send(Some(t));
}

pub async fn wait_result<T>(mut rx: UnboundedReceiver<Option<T>>) -> T {
    let result = rx.next().await;
    match result {
        Some(Some(t)) => t,
        _ => panic!(),
    }
}
