// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

pub struct TimeoutJoinHandle<T> {
    handle: thread::JoinHandle<T>,
    signal: Receiver<()>,
}

impl<T> TimeoutJoinHandle<T> {
    pub fn join(self, timeout: Duration) -> Result<T, Self> {
        if self.signal.recv_timeout(timeout).is_err() {
            return Err(self);
        }
        Ok(self.handle.join().unwrap())
    }
}

pub fn spawn<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(f: F) -> TimeoutJoinHandle<T> {
    let (send, recv) = channel();
    let t = thread::spawn(move || {
        let x = f();
        send.send(()).unwrap();
        x
    });
    TimeoutJoinHandle {
        handle: t,
        signal: recv,
    }
}
