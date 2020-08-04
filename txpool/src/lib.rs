// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
#[macro_use]
extern crate prometheus;
extern crate transaction_pool as tx_pool;

use actix::prelude::*;
use common_crypto::hash::HashValue;
use starcoin_bus::{Bus, BusActor};
use starcoin_config::NodeConfig;
use starcoin_txpool_api::TxnStatusFullEvent;
use std::{fmt::Debug, sync::Arc};
use storage::Store;
use tx_relay::{PeerTransactions, PropagateNewTransactions};

use counters::{TXPOOL_STATUS_GAUGE_VEC, TXPOOL_TXNS_GAUGE};
pub use pool::TxStatus;
use tx_pool_service_impl::Inner;
pub use tx_pool_service_impl::TxPoolService;

mod counters;
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
#[cfg(test)]
pub mod test_helper;
mod tx_pool_service_impl;

#[derive(Clone, Debug)]
pub struct TxPool {
    inner: Inner,
    addr: actix::Addr<TxPoolActor>,
}

impl TxPool {
    // TODO: use static dispatching instead of dynamic dispatch for storage instance.
    pub fn start(
        node_config: Arc<NodeConfig>,
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
        let service = TxPoolService::new(node_config, storage, best_block_header);
        let inner = service.get_inner();
        let pool = TxPoolActor::new(inner.clone(), bus);
        let pool_addr = pool.start();
        Self {
            inner,
            addr: pool_addr,
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
        // subscribe txn relayer peer txns
        let myself = ctx.address().recipient::<PeerTransactions>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe txn relayer message, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);

        ctx.add_stream(self.inner.subscribe_txns());

        info!("txn pool started");
    }
}
/// Listen to txn status, and propagate to remote peers if necessary.
impl StreamHandler<TxnStatusFullEvent> for TxPoolActor {
    fn handle(&mut self, item: TxnStatusFullEvent, ctx: &mut Context<Self>) {
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
            .broadcast(PropagateNewTransactions::new(txns))
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

impl actix::Handler<PeerTransactions> for TxPoolActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: PeerTransactions,
        _ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        // JUST need to keep at most once delivery.
        let txns = msg.peer_transactions();
        let _ = self.inner.import_txns(txns);
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
