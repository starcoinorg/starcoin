// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.

use anyhow::{format_err, Result};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use futures::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
pub use stest_macro::test;
pub use tokio::{runtime::Runtime, task::LocalSet};

pub mod actix_export {
    pub use actix_rt::*;
}

pub fn init_test_logger() {
    starcoin_logger::init_for_test();
}

pub fn timeout<F, T>(timeout: u64, f: F, tx: Sender<Result<T>>)
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let handle = timeout_join_handler::spawn(f);
    let result = handle
        .join(Duration::from_secs(timeout))
        .map_err(|e| anyhow::anyhow!("{}", e));
    let _ = tx.send(result);
}

pub fn wait_channel<T>(rx: Receiver<Result<T>>) -> T {
    let result = rx.recv();
    match result {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => panic!("test failed: {:?}", e),
        _ => panic!("test receiver error"),
    }
}

pub fn make_channel<T>() -> (UnboundedSender<Result<T>>, UnboundedReceiver<Result<T>>) {
    unbounded()
}

pub async fn timeout_future<T>(timeout: u64, tx: UnboundedSender<Result<T>>) {
    actix::clock::delay_for(Duration::from_secs(timeout)).await;
    let _ = tx.unbounded_send(Err(format_err!(
        "test timeout for wait {} seconds",
        timeout
    )));
}

pub async fn test_future<F, T>(f: F, tx: UnboundedSender<Result<T>>)
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let join = tokio::task::spawn_local(f);
    let t = join.await;
    let _ = tx.unbounded_send(t.map_err(Into::<anyhow::Error>::into));
}

pub async fn wait_result<T>(mut rx: UnboundedReceiver<Result<T>>) -> T {
    let result = rx.next().await;
    actix_rt::System::current().stop();
    match result {
        Some(Ok(t)) => t,
        Some(Err(e)) => panic!("test fail: {:?}", e),
        None => panic!("test timeout for wait result"),
    }
}
