// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Arc};

use futures::{
    channel::mpsc::{Receiver, Sender},
    sink::SinkExt,
    task::{Context, Poll},
    Future, Stream,
};

use anyhow::*;
use crypto::HashValue;
use futures::lock::Mutex;

use std::pin::Pin;

pub struct MessageFuture<T> {
    rx: Receiver<Result<T>>,
}

impl<T> MessageFuture<T> {
    pub fn new(rx: Receiver<Result<T>>) -> Self {
        Self { rx }
    }
}

impl<T> Future for MessageFuture<T> {
    type Output = Result<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        while let Poll::Ready(v) = Pin::new(&mut self.rx).poll_next(cx) {
            match v {
                Some(v) => match v {
                    Ok(v) => {
                        return Poll::Ready(Ok(v));
                    }
                    Err(e) => {
                        return Poll::Ready(Err(e));
                    }
                },
                None => {
                    warn!("no data,return timeout");
                    return Poll::Ready(Err(anyhow!("future time out")));
                }
            }
        }
        return Poll::Pending;
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

    pub async fn add_future(&self, hash: HashValue, sender: Sender<Result<T>>) {
        self.tx_map
            .lock()
            .await
            .entry(hash)
            .or_insert(sender.clone());
    }

    pub async fn send_response(&self, hash: HashValue, value: T) -> Result<()> {
        let mut tx_map = self.tx_map.lock().await;
        match tx_map.get(&hash) {
            Some(tx) => {
                match tx.clone().send(Ok(value)).await {
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

    pub async fn remove_future(&self, hash: HashValue) {
        let mut tx_map = self.tx_map.lock().await;
        match tx_map.get(&hash) {
            Some(tx) => {
                info!("future time out,hash is {:?}", hash);
                tx.clone()
                    .send(Err(anyhow!("future time out")))
                    .await
                    .unwrap();
                tx_map.remove(&hash);
            }
            _ => info!("tx hash {} not in map,timeout is not necessary", hash),
        }
    }
}
