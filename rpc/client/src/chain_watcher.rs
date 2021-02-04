// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::pubsub_client::PubSubClient;
use actix::prelude::*;
use actix::AsyncContext;
use futures::channel::oneshot;
use jsonrpc_core_client::RpcError;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::{BlockHeaderView, BlockView};
use starcoin_types::block::BlockNumber;
use std::collections::HashMap;
use std::time::Duration;

/// Block with only txn hashes.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinHeadBlock {
    pub header: BlockHeaderView,
    pub txn_hashes: Vec<HashValue>,
}

impl From<BlockView> for ThinHeadBlock {
    fn from(view: BlockView) -> Self {
        ThinHeadBlock {
            header: view.header,
            txn_hashes: view.body.txn_hashes(),
        }
    }
}

#[derive(Debug)]
pub struct ChainWatcher {
    watched_blocks: HashMap<BlockNumber, Vec<Responder>>,
    watched_txns: HashMap<HashValue, Responder>,
}
impl ChainWatcher {
    pub fn launch() -> Addr<Self> {
        let actor = Self {
            watched_txns: Default::default(),
            watched_blocks: Default::default(),
        };
        actor.start()
    }

    fn start_subscribe(&mut self, client: PubSubClient, ctx: &mut Context<Self>) {
        let inner_client = client.clone();
        async move { inner_client.subscribe_new_block().await }
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(s) => {
                        ctx.add_stream(s);
                    }
                    Err(e) => {
                        // TODO: figure out why this error cannot printed.
                        error!("fail to subscribe new block event, err: {}", &e);
                        ctx.notify_later(StartSubscribe { client }, Duration::from_secs(1));
                    }
                }
                async {}.into_actor(act)
            })
            .spawn(ctx);
    }
}

impl Actor for ChainWatcher {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("ChainWater actor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("ChainWater actor stopped");
    }
}

pub(crate) struct StartSubscribe {
    pub(crate) client: PubSubClient,
}

impl Message for StartSubscribe {
    type Result = ();
}

impl Handler<StartSubscribe> for ChainWatcher {
    type Result = ();

    fn handle(&mut self, msg: StartSubscribe, ctx: &mut Self::Context) {
        self.start_subscribe(msg.client, ctx)
    }
}

pub type WatchResult = Result<ThinHeadBlock, anyhow::Error>;
type Responder = oneshot::Sender<WatchResult>;

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct WatchBlock(pub BlockNumber);

impl Message for WatchBlock {
    type Result = oneshot::Receiver<WatchResult>;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct WatchTxn {
    pub txn_hash: HashValue,
}

impl Message for WatchTxn {
    type Result = oneshot::Receiver<WatchResult>;
}

impl Handler<WatchBlock> for ChainWatcher {
    type Result = MessageResult<WatchBlock>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: WatchBlock, _ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();
        self.watched_blocks
            .entry(msg.0)
            .or_insert_with(Vec::new)
            .push(tx);
        MessageResult(rx)
    }
}

impl Handler<WatchTxn> for ChainWatcher {
    type Result = MessageResult<WatchTxn>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: WatchTxn, _ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();
        self.watched_txns.entry(msg.txn_hash).or_insert(tx);
        MessageResult(rx)
    }
}

type BlockEvent = Result<BlockView, RpcError>;
impl actix::StreamHandler<BlockEvent> for ChainWatcher {
    fn handle(&mut self, item: BlockEvent, _ctx: &mut Self::Context) {
        match item {
            Ok(b) => {
                let b: ThinHeadBlock = b.into();
                if let Some(responders) = self.watched_blocks.remove(&b.header.number.0) {
                    for r in responders {
                        let _ = r.send(Ok(b.clone()));
                    }
                }
                for txn in &b.txn_hashes {
                    if let Some(r) = self.watched_txns.remove(txn) {
                        let _ = r.send(Ok(b.clone()));
                    }
                }
            }
            Err(e) => {
                for (_, responders) in self.watched_blocks.drain() {
                    for r in responders {
                        let e = anyhow::format_err!("{}", &e);
                        let _ = r.send(Err(e));
                    }
                }
                for (_, responder) in self.watched_txns.drain() {
                    let e = anyhow::format_err!("{}", &e);
                    let _ = responder.send(Err(e));
                }
            }
        }
    }
}

/// Try to stop a actor
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct StopWatcher;

impl Handler<StopWatcher> for ChainWatcher {
    type Result = ();

    fn handle(&mut self, _msg: StopWatcher, ctx: &mut Context<Self>) -> Self::Result {
        info!("Receive event to stop ChainWatcher.");
        ctx.stop();
        System::current().stop();
    }
}
