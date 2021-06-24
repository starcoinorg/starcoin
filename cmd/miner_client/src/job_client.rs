// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{JobClient, SealEvent};
use anyhow::Result;
use futures::stream::BoxStream;
use futures::{stream::StreamExt, Future, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_timer::Delay;
use logger::prelude::*;
use starcoin_config::{RealTimeService, TimeService};
use starcoin_rpc_client::RpcClient;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::MintBlockEvent;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct JobRpcClient {
    rpc_client: Arc<RpcClient>,
    seal_sender: UnboundedSender<(Vec<u8>, u32, BlockHeaderExtra)>,
    time_service: Arc<dyn TimeService>,
}

impl JobRpcClient {
    pub fn new(rpc_client: RpcClient) -> Self {
        let rpc_client = Arc::new(rpc_client);
        let seal_client = rpc_client.clone();
        let (seal_sender, mut seal_receiver) = unbounded::<(Vec<u8>, u32, BlockHeaderExtra)>();
        let fut = async move {
            while let Some((minting_blob, nonce, extra)) = seal_receiver.next().await {
                if let Err(e) = seal_client.miner_submit(
                    hex::encode(minting_blob),
                    nonce,
                    hex::encode(extra.to_vec()),
                ) {
                    warn!("Submit seal error: {}", e);
                    Delay::new(Duration::from_secs(1)).await;
                }
            }
        };
        Self::spawn(fut);
        Self {
            rpc_client,
            seal_sender,
            time_service: Arc::new(RealTimeService::new()),
        }
    }

    fn forward_mint_block_stream(&self) -> BoxStream<'static, MintBlockEvent> {
        let (sender, receiver) = unbounded();
        let client = self.rpc_client.clone();
        let fut = async move {
            // use a loop to retry subscribe event when connection error.
            loop {
                match client.subscribe_new_mint_blocks() {
                    Ok(stream) => {
                        let mut stream = stream.into_stream();
                        while let Some(item) = stream.next().await {
                            match item {
                                Ok(event) => {
                                    info!(
                                        "Receive mint event, minting_blob: {}, difficulty: {}",
                                        hex::encode(&event.minting_blob),
                                        event.difficulty
                                    );
                                    let _ = sender.unbounded_send(event);
                                }
                                Err(e) => {
                                    error!("Receive error event:{}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Subscribe new blocks event error: {}, retry later.", e);
                        Delay::new(Duration::from_secs(1)).await
                    }
                }
            }
        };
        Self::spawn(fut);
        receiver.boxed()
    }

    fn spawn<F>(fut: F)
    where
        F: Future + Send + 'static,
    {
        // if we use async spawn, RpcClient will panic when try reconnection.
        // refactor this after RpcClient refactor to async.
        std::thread::spawn(move || {
            futures::executor::block_on(fut);
        });
        //async_std::task::spawn(fut);
    }
}

impl JobClient for JobRpcClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        Ok(self.forward_mint_block_stream())
    }

    fn submit_seal(&self, seal: SealEvent) -> Result<()> {
        let extra = match &seal.extra {
            None => BlockHeaderExtra::default(),
            Some(extra) => extra.extra,
        };
        self.seal_sender
            .unbounded_send((seal.minting_blob, seal.nonce, extra))?;
        Ok(())
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}
