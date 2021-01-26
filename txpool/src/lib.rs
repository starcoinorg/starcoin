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
use network_api::messages::PeerTransactionsMessage;
pub use pool::TxStatus;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_state_api::AccountStateReader;
use starcoin_txpool_api::{PropagateTransactions, TxnStatusFullEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use storage::{BlockStore, Storage};
use tx_pool_service_impl::Inner;
pub use tx_pool_service_impl::TxPoolService;
use types::{
    sync_status::SyncStatus, system_events::SyncStatusChangeEvent,
    transaction::SignedUserTransaction,
};

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
    next_propagation_ready: Arc<AtomicBool>,
    sync_status: Option<SyncStatus>,
}

impl std::fmt::Debug for TxPoolActorService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pool: {:?}", &self.inner,)
    }
}

const MIN_TXN_TO_PROPAGATE: usize = 256;
const PROPAGATE_FOR_BLOCKS: u64 = 4;

impl TxPoolActorService {
    fn new(inner: Inner) -> Self {
        Self {
            inner,
            sync_status: None,
            next_propagation_ready: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }

    fn transactions_to_propagate(&self) -> Result<Vec<SignedUserTransaction>> {
        let statedb = self.inner.get_chain_reader();
        let reader = AccountStateReader::new(&statedb);
        let block_gas_limit = reader.get_epoch()?.block_gas_limit();
        // TODO: fetch from a gas constants
        let min_tx_gas = 200;

        let max_len = std::cmp::max(
            MIN_TXN_TO_PROPAGATE,
            (block_gas_limit / min_tx_gas * PROPAGATE_FOR_BLOCKS) as usize,
        );
        let current_timestamp = reader.get_timestamp()?.seconds();
        Ok(self
            .inner
            .get_pending(max_len as u64, current_timestamp)
            .into_iter()
            .map(|t| t.signed().clone())
            .collect())
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
                .get_block_by_hash(startup_info.main)?
                .ok_or_else(|| {
                    format_err!(
                        "best block id {} should exists in storage",
                        startup_info.main
                    )
                })?;

            let best_block_header = best_block.into_inner().0;
            Ok(TxPoolService::new(node_config, storage, best_block_header))
        })?;
        Ok(Self::new(txpool_service.get_inner()))
    }
}
impl TxPoolActorService {
    fn try_propagate_txns(&self, ctx: &mut ServiceContext<Self>) {
        if self.next_propagation_ready.load(Ordering::Relaxed) {
            match self.transactions_to_propagate() {
                Err(e) => {
                    log::error!("txpool: fail to get txn to propagate, err: {}", &e)
                }
                Ok(txs) if !txs.is_empty() => {
                    if self
                        .next_propagation_ready
                        .compare_and_swap(true, false, Ordering::AcqRel)
                    {
                        let request =
                            PropagateTransactions::new(txs, self.next_propagation_ready.clone());
                        ctx.broadcast(request);
                    }
                }
                Ok(_) => {}
            }
        }
    }
}
impl ActorService for TxPoolActorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.add_stream(self.inner.subscribe_txns());

        // every x seconds, we tick a txn propagation.
        let myself = self.clone();
        let interval = self.inner.node_config.tx_pool.tx_propagate_interval();
        ctx.run_interval(Duration::from_secs(interval), move |ctx| {
            myself.try_propagate_txns(ctx)
        });

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        Ok(())
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for TxPoolActorService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

/// Listen to txn status, and propagate to remote peers if necessary.
impl EventHandler<Self, TxnStatusFullEvent> for TxPoolActorService {
    fn handle_event(&mut self, item: TxnStatusFullEvent, ctx: &mut ServiceContext<Self>) {
        // do metrics.
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

            for (_, s) in item.iter() {
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
            }
        }
        //TODO direct send broadcast message to network.
        self.try_propagate_txns(ctx);
    }
}

impl EventHandler<Self, PeerTransactionsMessage> for TxPoolActorService {
    fn handle_event(&mut self, msg: PeerTransactionsMessage, _ctx: &mut ServiceContext<Self>) {
        //TODO should filter msg an NetworkService
        if self.is_synced() {
            // JUST need to keep at most once delivery.
            let _ = self.inner.import_txns(msg.message.txns);
        } else {
            debug!("[txpool] Ignore PeerTransactions event because the node has not been synchronized yet.");
        }
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
