use crate::pubsub_client::PubSubClient;
use actix::prelude::*;
use actix::AsyncContext;
use futures03::channel::oneshot;
use futures03::compat::Stream01CompatExt;
use jsonrpc_core_client::RpcError;
pub use pubsub::ThinBlock;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::pubsub;
use starcoin_types::block::BlockNumber;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ChainWatcher {
    inner: PubSubClient,
    watched_blocks: HashMap<BlockNumber, Vec<Responder>>,
    watched_txns: HashMap<HashValue, Responder>,
}
impl ChainWatcher {
    pub fn launch(pubsub_client: PubSubClient) -> Addr<Self> {
        let actor = Self {
            inner: pubsub_client,
            watched_txns: Default::default(),
            watched_blocks: Default::default(),
        };
        actor.start()
    }
}

impl Actor for ChainWatcher {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let client = self.inner.clone();
        async move {
            client.subscribe_new_block().await
        }.into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(s) => {
                        ctx.add_stream(s.compat());
                    }
                    Err(e) => {
                        error!(target: "chain_watcher", "fail to subscribe new block event, err: {}", &e);
                        ctx.terminate();
                    }
                }
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}

pub type WatchResult = Result<pubsub::ThinBlock, anyhow::Error>;
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

type BlockEvent = Result<pubsub::ThinBlock, RpcError>;
impl actix::StreamHandler<BlockEvent> for ChainWatcher {
    fn handle(&mut self, item: BlockEvent, _ctx: &mut Self::Context) {
        match &item {
            Ok(b) => {
                if let Some(responders) = self.watched_blocks.remove(&b.header().number()) {
                    for r in responders {
                        let _ = r.send(Ok(b.clone()));
                    }
                }
                for txn in b.body() {
                    if let Some(r) = self.watched_txns.remove(txn) {
                        let _ = r.send(Ok(b.clone()));
                    }
                }
            }
            Err(e) => {
                // if any error happen in subscription, return error to client
                for (_, responders) in self.watched_blocks.drain() {
                    for r in responders {
                        let e = anyhow::format_err!("{}", e);
                        let _ = r.send(Err(e));
                    }
                }
                for (_, responder) in self.watched_txns.drain() {
                    let e = anyhow::format_err!("{}", e);
                    let _ = responder.send(Err(e));
                }
            }
        }
    }
    // fn finished(&mut self, ctx: &mut Self::Context) {}
}
