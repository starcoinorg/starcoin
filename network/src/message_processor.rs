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
use futures::lock::Mutex;

use std::cmp::Eq;
use std::hash::Hash;
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

    //FIXME
    #[allow(clippy::never_loop)]
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
        Poll::Pending
    }
}

#[derive(Clone)]
pub struct MessageProcessor<K, T> {
    tx_map: Arc<Mutex<HashMap<K, Sender<Result<T>>>>>,
}

impl<K, T> MessageProcessor<K, T>
where
    K: Send + Sync + Hash + Eq + 'static,
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            tx_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_future(&self, id: K, sender: Sender<Result<T>>) {
        self.tx_map
            .lock()
            .await
            .entry(id)
            .or_insert_with(|| sender.clone());
    }

    pub async fn send_response(&self, id: K, value: T) -> Result<()> {
        let mut tx_map = self.tx_map.lock().await;
        match tx_map.get(&id) {
            Some(tx) => {
                match tx.clone().send(Ok(value)).await {
                    Ok(_new_tx) => {
                        info!("send message succ");
                        tx_map.remove(&id);
                    }
                    Err(_) => warn!("send message error"),
                };
            }
            _ => info!("tx id  not in map"),
        }
        Ok(())
    }
    //
    pub async fn remove_future(&self, id: K) -> bool {
        let mut tx_map = self.tx_map.lock().await;
        if let Some(tx) = tx_map.get(&id) {
            tx.clone()
                .send(Err(anyhow!("future time out")))
                .await
                .unwrap();
            tx_map.remove(&id);
            // if find tx ,means timeout
            return true;
        }
        false
    }
}
