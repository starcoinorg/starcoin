// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#[macro_use]
extern crate async_trait;
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
#[macro_use]
extern crate prometheus;
extern crate transaction_pool as tx_pool;

use actix::prelude::*;
use anyhow::Result;
use common_crypto::hash::HashValue;
use futures_channel::mpsc;
use starcoin_bus::{Bus, BusActor};
use starcoin_config::TxPoolConfig;
use starcoin_txpool_api::TxPoolAsyncService;
use std::{fmt::Debug, sync::Arc};
use storage::Store;
use tx_relay::{PeerTransactions, PropagateNewTransactions};
use types::{
    account_address::AccountAddress, system_events::NewHeadBlock, transaction,
    transaction::SignedUserTransaction,
};

use counters::TXPOOL_SERVICE_HISTOGRAM;
use counters::{TXPOOL_STATUS_GAUGE_VEC, TXPOOL_TXNS_GAUGE};
pub use pool::TxStatus;
use tx_pool_service_impl::Inner;
pub use tx_pool_service_impl::TxPoolService;

mod counters;
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
pub mod test_helper;
mod tx_pool_service_impl;

#[derive(Clone, Debug)]
pub struct TxPool {
    inner: Inner,
    addr: actix::Addr<TxPoolActor>,
}

impl TxPool {
    pub fn start(
        pool_config: TxPoolConfig,
        storage: Arc<dyn Store>,
        best_block_hash: HashValue,
        bus: actix::Addr<BusActor>,
    ) -> Self {
        let best_block = match storage.get_block_by_hash(best_block_hash) {
            Err(e) => panic!("fail to read storage, {}", e),
            Ok(None) => panic!(
                "best block id {} should exists in storage",
                &best_block_hash
            ),
            Ok(Some(block)) => block,
        };
        let best_block_header = best_block.into_inner().0;
        let service = TxPoolService::new(pool_config, storage, best_block_header);
        let inner = service.get_inner();
        let pool = TxPoolActor::new(inner.clone(), bus);
        let pool_addr = pool.start();
        Self {
            inner,
            addr: pool_addr,
        }
    }

    #[cfg(test)]
    pub fn start_with_best_block_header(
        storage: Arc<dyn Store>,
        best_block_header: types::block::BlockHeader,
        bus: actix::Addr<BusActor>,
    ) -> Self {
        let service = TxPoolService::new(TxPoolConfig::default(), storage, best_block_header);
        let inner = service.get_inner();
        let pool = TxPoolActor::new(inner.clone(), bus);
        let pool_addr = pool.start();
        Self {
            inner,
            addr: pool_addr,
        }
    }

    pub fn get_async_service(&self) -> TxPoolRef {
        TxPoolRef {
            addr: self.addr.clone(),
        }
    }

    pub fn get_service(&self) -> TxPoolService {
        TxPoolService::from_inner(self.inner.clone())
    }
}

#[derive(Clone, Debug)]
pub struct TxPoolRef {
    addr: actix::Addr<TxPoolActor>,
}

#[async_trait]
impl TxPoolAsyncService for TxPoolRef {
    async fn add(self, txn: SignedUserTransaction) -> Result<bool> {
        let mut result = self.add_txns(vec![txn]).await?;
        Ok(result.pop().unwrap().is_ok())
    }

    async fn add_txns(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["add_txns"])
            .start_timer();
        let result = self.addr.send(ImportTxns { txns }).await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    async fn remove_txn(
        self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Result<Option<SignedUserTransaction>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["remove_txn"])
            .start_timer();
        let result = self
            .addr
            .send(RemoveTxn {
                txn_hash,
                is_invalid,
            })
            .await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r.map(|v| v.signed().clone())),
        }
    }

    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["get_pending_txns"])
            .start_timer();
        let result = self
            .addr
            .send(GetPendingTxns {
                max_len: max_len.unwrap_or(u64::max_value()),
            })
            .await;
        timer.observe_duration();
        match result {
            Ok(r) => Ok(r.into_iter().map(|t| t.signed().clone()).collect()),
            Err(e) => Err(e.into()),
        }
    }
    async fn next_sequence_number(self, address: AccountAddress) -> Result<Option<u64>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["next_sequence_number"])
            .start_timer();
        let result = self.addr.send(NextSequenceNumber { address }).await;
        timer.observe_duration();
        result.map_err(|e| e.into())
    }

    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["subscribe_txns"])
            .start_timer();
        let result = self.addr.send(SubscribeTxns).await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    /// when new block happened in chain, use this to notify txn pool
    /// the `HashValue` of `enacted`/`retracted` is the hash of txns.
    /// enacted: the txns which enter into main chain.
    /// retracted: the txns which is rollbacked.
    async fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["rollback"])
            .start_timer();
        let result = self.addr.send(ChainNewBlock { enacted, retracted }).await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r?),
        }
    }
}

type TxnQueue = pool::TransactionQueue;
#[derive(Clone)]
pub(crate) struct TxPoolActor {
    inner: Inner,
    bus: actix::Addr<BusActor>,
}
impl std::fmt::Debug for TxPoolActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pool: {:?}, bus: {:?}",
            &self.inner,
            self.bus.connected()
        )
    }
}

impl TxPoolActor {
    pub fn new(inner: Inner, bus: actix::Addr<BusActor>) -> Self {
        Self { bus, inner }
    }

    pub fn launch(self) -> TxPoolRef {
        let addr = self.start();
        TxPoolRef { addr }
    }
}

impl actix::Actor for TxPoolActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // subscribe system block event
        let myself = ctx.address().recipient::<NewHeadBlock>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe system events, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);

        // subscribe txn relay peer txns
        let myself = ctx.address().recipient::<PeerTransactions>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe txn relay message, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);

        ctx.add_stream(self.inner.subscribe_txns());

        info!("txn pool started");
    }
}
type TxnStatusEvent = Arc<Vec<(HashValue, TxStatus)>>;
/// Listen to txn status, and propagate to remote peers if necessary.
impl StreamHandler<TxnStatusEvent> for TxPoolActor {
    fn handle(&mut self, item: TxnStatusEvent, ctx: &mut Context<Self>) {
        {
            let status = self.inner.pool_status().status;
            let mem_usage = status.mem_usage;
            let senders = status.senders;
            let txn_count = status.transaction_count;
            TXPOOL_STATUS_GAUGE_VEC
                .with_label_values(&["mem_usage"])
                .set(mem_usage as i64);
            TXPOOL_STATUS_GAUGE_VEC
                .with_label_values(&["senders"])
                .set(senders as i64);
            TXPOOL_STATUS_GAUGE_VEC
                .with_label_values(&["count"])
                .set(txn_count as i64);
        }
        // TODO: need peer info to do more accurate sending.
        let mut txns = vec![];
        for (h, s) in item.iter() {
            match *s {
                TxStatus::Added => {
                    TXPOOL_TXNS_GAUGE.inc();
                }
                TxStatus::Rejected => {}
                _ => {
                    TXPOOL_TXNS_GAUGE.dec();
                }
            }

            if *s != TxStatus::Added {
                continue;
            }

            if let Some(txn) = self.inner.queue().find(h) {
                txns.push(txn.signed().clone());
            }
        }
        if txns.is_empty() {
            return;
        }
        self.bus
            .clone()
            .broadcast(PropagateNewTransactions::from(txns))
            .into_actor(self)
            .then(|res, act, _ctx| {
                if let Err(e) = res {
                    error!("fail to emit propagate new txn event, err: {}", &e);
                }
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}

impl actix::Handler<NewHeadBlock> for TxPoolActor {
    type Result = ();

    fn handle(&mut self, msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        let NewHeadBlock(block) = msg;
        self.inner
            .notify_new_chain_header(block.get_block().header().clone());
        self.inner.cull()
    }
}

impl actix::Handler<PeerTransactions> for TxPoolActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: PeerTransactions,
        ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        // JUST need to keep at most once delivery.
        let txns = msg.peer_transactions();
        ctx.notify(ImportTxns { txns })
    }
}

pub(crate) struct ImportTxns {
    pub(crate) txns: Vec<transaction::SignedUserTransaction>,
}

impl actix::Message for ImportTxns {
    type Result = Vec<Result<(), transaction::TransactionError>>;
}
impl actix::Handler<ImportTxns> for TxPoolActor {
    type Result = actix::MessageResult<ImportTxns>;

    fn handle(&mut self, msg: ImportTxns, _ctx: &mut Self::Context) -> Self::Result {
        let ImportTxns { txns } = msg;
        actix::MessageResult(self.inner.import_txns(txns))
    }
}

pub(crate) struct RemoveTxn {
    pub(crate) txn_hash: HashValue,
    pub(crate) is_invalid: bool,
}
impl actix::Message for RemoveTxn {
    type Result = Option<Arc<pool::VerifiedTransaction>>;
}
impl actix::Handler<RemoveTxn> for TxPoolActor {
    type Result = actix::MessageResult<RemoveTxn>;

    fn handle(&mut self, msg: RemoveTxn, _ctx: &mut Self::Context) -> Self::Result {
        let RemoveTxn {
            txn_hash,
            is_invalid,
        } = msg;
        actix::MessageResult(self.inner.remove_txn(txn_hash, is_invalid))
    }
}

pub(crate) struct GetPendingTxns {
    pub(crate) max_len: u64,
}

impl actix::Message for GetPendingTxns {
    type Result = Vec<Arc<pool::VerifiedTransaction>>;
}

impl actix::Handler<GetPendingTxns> for TxPoolActor {
    type Result = actix::MessageResult<GetPendingTxns>;

    fn handle(&mut self, msg: GetPendingTxns, _ctx: &mut Self::Context) -> Self::Result {
        let GetPendingTxns { max_len } = msg;
        actix::MessageResult(self.inner.get_pending(max_len))
    }
}

pub(crate) struct NextSequenceNumber {
    pub(crate) address: AccountAddress,
}

impl actix::Message for NextSequenceNumber {
    type Result = Option<u64>;
}

impl actix::Handler<NextSequenceNumber> for TxPoolActor {
    type Result = actix::MessageResult<NextSequenceNumber>;

    fn handle(&mut self, msg: NextSequenceNumber, _ctx: &mut Self::Context) -> Self::Result {
        let NextSequenceNumber { address } = msg;
        actix::MessageResult(self.inner.next_sequence_number(address))
    }
}

pub(crate) struct SubscribeTxns;
impl actix::Message for SubscribeTxns {
    type Result = mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>;
}

impl actix::Handler<SubscribeTxns> for TxPoolActor {
    type Result = actix::MessageResult<SubscribeTxns>;

    fn handle(&mut self, _: SubscribeTxns, _ctx: &mut Self::Context) -> Self::Result {
        actix::MessageResult(self.inner.subscribe_txns())
    }
}

pub(crate) struct ChainNewBlock {
    pub(crate) enacted: Vec<SignedUserTransaction>,
    pub(crate) retracted: Vec<SignedUserTransaction>,
}
impl actix::Message for ChainNewBlock {
    type Result = Result<()>;
}
impl actix::Handler<ChainNewBlock> for TxPoolActor {
    type Result = <ChainNewBlock as actix::Message>::Result;

    fn handle(&mut self, msg: ChainNewBlock, _ctx: &mut Self::Context) -> Self::Result {
        let ChainNewBlock { enacted, retracted } = msg;
        self.inner.chain_new_block(enacted, retracted)
    }
}

#[cfg(test)]
mod test_sync_and_send {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_static<T: 'static>() {}
    #[test]
    fn test_sync_and_send() {
        assert_send::<super::TxPoolActor>();
        assert_sync::<super::TxPoolActor>();
        assert_static::<super::TxPoolActor>();
    }
}
