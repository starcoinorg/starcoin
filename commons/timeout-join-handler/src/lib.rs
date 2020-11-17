// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error)]
pub enum ThreadJoinError<T> {
    #[error("Thread join timeout")]
    Timeout(TimeoutJoinHandle<T>),
    #[error("Thread panic: {0}")]
    Panic(&'static str),
    #[error("Thread return unknown error")]
    Unknown(Box<dyn Any + Send>),
}

impl<T> std::fmt::Debug for ThreadJoinError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ThreadJoinError::Timeout(_) => "timeout".to_string(),
            ThreadJoinError::Panic(msg) => format!("panic({})", msg),
            ThreadJoinError::Unknown(_) => format!("unknown"),
        };
        write!(f, "ThreadJoinError({})", msg)
    }
}

impl<T> ThreadJoinError<T> {
    pub fn is_timeout(&self) -> bool {
        matches!(self, ThreadJoinError::Timeout(_))
    }

    pub fn is_panic(&self) -> bool {
        matches!(self, ThreadJoinError::Panic(_))
    }

    pub fn into_handle(self) -> Option<TimeoutJoinHandle<T>> {
        if let ThreadJoinError::Timeout(handle) = self {
            Some(handle)
        } else {
            None
        }
    }

    pub fn panic_message(&self) -> Option<&'static str> {
        if let ThreadJoinError::Panic(msg) = self {
            Some(msg)
        } else {
            None
        }
    }
}

pub struct TimeoutJoinHandle<T> {
    handle: thread::JoinHandle<T>,
    signal: Receiver<()>,
}

impl<T> std::fmt::Debug for TimeoutJoinHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("TimeoutJoinHandle { .. }")
    }
}

impl<T> TimeoutJoinHandle<T> {
    /// if thread join wait timeout, return handle self, otherwise return thread join result.
    pub fn join(self, timeout: Duration) -> Result<T, ThreadJoinError<T>> {
        if let Err(RecvTimeoutError::Timeout) = self.signal.recv_timeout(timeout) {
            return Err(ThreadJoinError::Timeout(self));
        }
        self.handle.join().map_err(|e| {
            if let Some(e) = e.downcast_ref::<&'static str>() {
                ThreadJoinError::Panic(e)
            } else {
                ThreadJoinError::Unknown(e)
            }
        })
    }
}

pub fn spawn<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(f: F) -> TimeoutJoinHandle<T> {
    let (send, recv) = channel();
    let t = thread::spawn(move || {
        let x = f();
        //ignore send error.
        let _e = send.send(());
        x
    });
    TimeoutJoinHandle {
        handle: t,
        signal: recv,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_ok() {
        let handle = spawn(|| {
            thread::sleep(Duration::from_millis(100));
            let result: Result<u64, anyhow::Error> = Ok(1);
            result
        });
        let result = handle.join(Duration::from_millis(1000));
        assert!(result.is_ok());
        assert_eq!(1, result.unwrap().unwrap());
    }

    #[test]
    fn test_error_in_thread() {
        let handle = spawn(|| {
            thread::sleep(Duration::from_millis(100));
            let result: Result<(), anyhow::Error> = Err(anyhow!("anyhow error"));
            result
        });
        let result = handle.join(Duration::from_millis(1000));
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_timeout() {
        let handle = spawn(|| {
            thread::sleep(Duration::from_secs(1));
        });
        let result = handle.join(Duration::from_millis(100));
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.is_timeout());
        let handle = error.into_handle().unwrap();
        let result = handle.join(Duration::from_secs(2));
        assert!(result.is_ok());
    }

    #[test]
    fn test_panic() {
        let handle = spawn(|| {
            panic!("test thread panic");
        });
        let result = handle.join(Duration::from_secs(2));
        let error = result.err().unwrap();
        assert!(error.is_panic());
        assert_eq!(error.panic_message().unwrap(), "test thread panic");
    }
}
