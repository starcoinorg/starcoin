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

use anyhow::{format_err, Result};
use counters::{TXPOOL_STATUS_GAUGE_VEC, TXPOOL_TXNS_GAUGE};
use starcoin_config::NodeConfig;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_txpool_api::TxnStatusFullEvent;
use std::sync::Arc;
use storage::{BlockStore, Storage};
use tx_pool_service_impl::Inner;
use tx_relay::{PeerTransactions, PropagateNewTransactions};

pub use pool::TxStatus;
pub use tx_pool_service_impl::TxPoolService;

mod counters;
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
mod tx_pool_service_impl;

//TODO refactor TxPoolService and rename.
#[derive(Clone)]
pub struct TxPoolActorService {
    inner: Inner,
}

impl std::fmt::Debug for TxPoolActorService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pool: {:?}", &self.inner,)
    }
}

impl TxPoolActorService {
    fn new(inner: Inner) -> Self {
        Self { inner }
    }
}

impl ServiceFactory<Self> for TxPoolActorService {
    fn create(ctx: &mut ServiceContext<TxPoolActorService>) -> Result<TxPoolActorService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let node_config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let txpool_service = ctx.get_shared_or_put(|| {
            let startup_info = storage
                .get_startup_info()?
                .ok_or_else(|| format_err!("StartupInfo should exist when service init."))?;
            let best_block = storage
                .get_block_by_hash(startup_info.master)?
                .ok_or_else(|| {
                    format_err!(
                        "best block id {} should exists in storage",
                        startup_info.master
                    )
                })?;

            let best_block_header = best_block.into_inner().0;
            Ok(TxPoolService::new(node_config, storage, best_block_header))
        })?;
        Ok(Self::new(txpool_service.get_inner()))
    }
}

impl ActorService for TxPoolActorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<PeerTransactions>();
        ctx.add_stream(self.inner.subscribe_txns());
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PeerTransactions>();
        Ok(())
    }
}

/// Listen to txn status, and propagate to remote peers if necessary.
impl EventHandler<Self, TxnStatusFullEvent> for TxPoolActorService {
    fn handle_event(&mut self, item: TxnStatusFullEvent, ctx: &mut ServiceContext<Self>) {
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
        ctx.broadcast(PropagateNewTransactions::new(txns));
    }
}

impl EventHandler<Self, PeerTransactions> for TxPoolActorService {
    fn handle_event(&mut self, msg: PeerTransactions, _ctx: &mut ServiceContext<Self>) {
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
        assert_send::<super::TxPoolActorService>();
        assert_sync::<super::TxPoolActorService>();
        assert_static::<super::TxPoolActorService>();
    }
}
