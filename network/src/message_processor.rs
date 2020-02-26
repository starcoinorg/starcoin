// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{
    sink::Sink,
    sync::mpsc::{Receiver, Sender},
    Async, Future, Poll, Stream,
};

use anyhow::{Error, Result};
use crypto::hash::HashValue;

pub struct MessageFuture<T> {
    rx: Receiver<Result<T>>,
}

impl<T> MessageFuture<T> {
    pub fn new(rx: Receiver<Result<T>>) -> Self {
        Self { rx }
    }
}

impl<T> Future for MessageFuture<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(v) = self.rx.poll().unwrap() {
            match v {
                Some(v) => match v {
                    Ok(v) => {
                        return Ok(Async::Ready(v));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                },
                None => {
                    warn!("no data,return timeout");
                    return Err(std::io::Error::from(std::io::ErrorKind::TimedOut).into());
                }
            }
        }
        return Ok(Async::NotReady);
    }
}

#[derive(Clone)]
pub struct MessageProcessor<T> {
    tx_map: Arc<Mutex<HashMap<HashValue, Sender<Result<T>>>>>,
}

impl<T> MessageProcessor<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            tx_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_future(&self, hash: HashValue, sender: Sender<Result<T>>) {
        self.tx_map
            .lock()
            .unwrap()
            .entry(hash)
            .or_insert(sender.clone());
    }

    pub fn send_response(&self, hash: HashValue, value: T) -> Result<()> {
        let mut tx_map = self.tx_map.lock().unwrap();
        match tx_map.get(&hash) {
            Some(tx) => {
                match tx.clone().send(Ok(value)).wait() {
                    Ok(_new_tx) => {
                        info!("send message succ");
                        tx_map.remove(&hash);
                    }
                    Err(_) => warn!("send message error"),
                };
            }
            _ => info!("tx hash {} not in map", hash),
        }
        Ok(())
    }

    pub fn remove_future(&self, hash: HashValue) {
        let mut tx_map = self.tx_map.lock().unwrap();
        match tx_map.get(&hash) {
            Some(_tx) => {
                info!("future time out,hash is {:?}", hash);
                tx_map.remove(&hash);
            }
            _ => info!("tx hash {} not in map,timeout is not necessary", hash),
        }
    }

    // pub fn future_error(&self, error_msg: ErrorMessage) -> Result<()> {
    //     let mut tx_map = self.tx_map.lock().unwrap();
    //     match tx_map.get(&error_msg.raw_transaction_hash) {
    //         Some(tx) => {
    //             tx.clone().send(Err(error_msg.error.into())).wait()?;
    //             tx_map.remove(&error_msg.raw_transaction_hash);
    //         }
    //         _ => info!(
    //             "tx hash {} not in map,error is not necessary",
    //             error_msg.raw_transaction_hash
    //         ),
    //     }
    //     Ok(())
    // }
}
