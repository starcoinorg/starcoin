// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
extern crate transaction_pool as tx_pool;

use anyhow::{bail, format_err, Result};
use futures_channel::mpsc;
use network_api::messages::PeerTransactionsMessage;
pub use pool::queue::Pool;
pub use pool::TxStatus;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_state_api::{AccountStateReader, StateReaderExt};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_txpool_api::{
    PropagateTransactions, TxPoolStatus, TxPoolSyncService, TxnStatusFullEvent,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::Block;
use starcoin_types::transaction;
use starcoin_types::{
    sync_status::SyncStatus, system_events::SyncStatusChangeEvent,
    transaction::SignedUserTransaction,
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tx_pool_service_impl::Inner;
pub use tx_pool_service_impl::TxPoolService;

mod metrics;
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
mod tx_pool_service_impl;
//TODO refactor TxPoolService and rename.
#[derive(Clone)]
pub struct TxPoolActorService {
    // inner: Inner,
    new_txs_received: Arc<AtomicBool>,
    sync_status: Option<SyncStatus>,
}

impl std::fmt::Debug for TxPoolActorService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pool: ",)
    }
}

const MIN_TXN_TO_PROPAGATE: usize = 256;
const PROPAGATE_FOR_BLOCKS: u64 = 4;

impl TxPoolActorService {
    fn new() -> Self {
        Self {
            // inner,
            sync_status: None,
            new_txs_received: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }

    fn transactions_to_propagate(&self) -> Result<Vec<SignedUserTransaction>> {
        Ok(vec![])
        // let statedb = self.inner.get_chain_reader();
        // let reader = AccountStateReader::new(&statedb);

        // // TODO: fetch from a gas constants
        // //TODO optimize broadcast txn by hash, then calculate max length by block gas limit
        // // currently use a small size for reduce broadcast message size.
        // // let block_gas_limit = reader.get_epoch()?.block_gas_limit();
        // //let min_tx_gas = 200;
        // // let max_len = std::cmp::max(
        // //     MIN_TXN_TO_PROPAGATE,
        // //     (block_gas_limit / min_tx_gas * PROPAGATE_FOR_BLOCKS) as usize,
        // // );
        // let max_len = 100;
        // let current_timestamp = reader.get_timestamp()?.seconds();
        // Ok(self
        //     .inner
        //     .get_pending(max_len, current_timestamp)
        //     .into_iter()
        //     .map(|t| t.signed().clone())
        //     .collect())
    }
}

impl ServiceFactory<Self> for TxPoolActorService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let node_config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
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
            Ok(MockTxPoolService::new(storage))
        })?;
        Ok(Self::new())
    }
}
impl TxPoolActorService {
    fn try_propagate_txns(&self, ctx: &mut ServiceContext<Self>) {
        // only propagate when new txns enter pool.
        if self.new_txs_received.load(Ordering::Relaxed) {
            match self.transactions_to_propagate() {
                Err(e) => {
                    log::error!("txpool: fail to get txn to propagate, err: {}", &e)
                }
                Ok(txs) if !txs.is_empty() => {
                    if self
                        .new_txs_received
                        .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
                        .unwrap_or_else(|x| x)
                    {
                        let request = PropagateTransactions::new(txs);
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
        // ctx.add_stream(self.inner.subscribe_txns());

        // every x seconds, we tick a txn propagation.
        // let myself = self.clone();
        // let interval = self.inner.node_config.tx_pool.tx_propagate_interval();
        // ctx.run_interval(Duration::from_secs(interval), move |ctx| {
        //     myself.try_propagate_txns(ctx)
        // });

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
    fn handle_event(&mut self, item: TxnStatusFullEvent, _ctx: &mut ServiceContext<Self>) {
        // do metrics.
        // if let Some(metrics) = self.inner.metrics.as_ref() {
        //     let status = self.inner.pool_status().status;
        //     let mem_usage = status.mem_usage;
        //     let senders = status.senders;
        //     let txn_count = status.transaction_count;

        //     metrics
        //         .txpool_status
        //         .with_label_values(&["mem_usage"])
        //         .set(mem_usage as u64);
        //     metrics
        //         .txpool_status
        //         .with_label_values(&["senders"])
        //         .set(senders as u64);
        //     metrics
        //         .txpool_status
        //         .with_label_values(&["count"])
        //         .set(txn_count as u64);
        // }
        // let mut has_new_txns = false;
        // for (_, s) in item.iter() {
        //     if let Some(metrics) = self.inner.metrics.as_ref() {
        //         metrics
        //             .txpool_txn_event_total
        //             .with_label_values(&[format!("{}", s).as_str()])
        //             .inc();
        //     }

        //     if *s == TxStatus::Added {
        //         has_new_txns = true;
        //     }
        // }
        // if has_new_txns {
        //     // notify txn-broadcaster.
        //     self.new_txs_received.store(true, Ordering::Relaxed);
        // }
    }
}

impl EventHandler<Self, PeerTransactionsMessage> for TxPoolActorService {
    fn handle_event(&mut self, msg: PeerTransactionsMessage, _ctx: &mut ServiceContext<Self>) {
        if self.is_synced() {
            // JUST need to keep at most once delivery.
            // let _ = self.inner.import_txns(msg.message.txns);
        } else {
            //TODO should keep txn in a buffer, then execute after sync finished.
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

#[derive(Clone)]
pub struct MockTxPoolService {
    pool: Arc<Mutex<VecDeque<SignedUserTransaction>>>,
    storage: Arc<dyn Store>,
}

impl MockTxPoolService {
    pub fn new(storage: Arc<dyn Store>) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            storage,
        }
    }

    pub fn new_with_txns(txns: Vec<SignedUserTransaction>, storage: Arc<dyn Store>) -> Self {
        Self {
            pool: Arc::new(Mutex::new(txns.into())),
            storage,
        }
    }

    pub fn verify_transaction(
        &self,
        _tx: SignedUserTransaction,
    ) -> Result<(), transaction::TransactionError> {
        Ok(())
    }

    pub fn get_store(&self) -> Arc<dyn Store> {
        self.storage.clone()
    }
}

impl TxPoolSyncService for MockTxPoolService {
    fn add_txns(
        &self,
        txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>> {
        if txns.is_empty() {
            return vec![Ok(())];
        }
        let id = txns[0].id();
        let len = txns.len();
        let mut results = vec![];
        let mut pool = match self.pool.lock() {
            Ok(pool) => pool,
            Err(e) => {
                results.resize_with(len, || Ok(()));
                return results;
            }
        };
        pool.append(&mut txns.into());
        results.resize_with(len, || Ok(()));
        results
    }

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    fn remove_txn(&self, txn_hash: HashValue, _is_invalid: bool) -> Option<SignedUserTransaction> {
        self.pool.lock().ok().and_then(|mut lock| {
            lock.iter()
                .position(|t| t.id() == txn_hash)
                .and_then(|i| lock.remove(i))
        })
    }

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(
        &self,
        max_len: Option<u64>,
        _now: Option<u64>,
    ) -> Vec<SignedUserTransaction> {
        match max_len {
            Some(max) => self
                .pool
                .lock()
                .ok()
                .and_then(|mut pool| Some(pool.drain(0..max as usize).collect::<Vec<_>>()))
                .unwrap(),
            None => self
                .pool
                .lock()
                .ok()
                .and_then(|mut pool| Some(pool.drain(..).collect::<Vec<_>>()))
                .unwrap(),
        }
    }

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, _address: AccountAddress) -> Option<u64> {
        None
    }

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent> {
        unimplemented!("not implemented yet")
    }
    fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>> {
        unimplemented!("not implemented yet")
    }

    fn chain_new_block(&self, enacted: Vec<Block>, retracted: Vec<Block>) -> Result<()> {
        if retracted.is_empty() {
            return Ok(());
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;

        let state_root = enacted.first().unwrap().header().state_root();
        let storage = self.storage.clone();
        let statedb = ChainStateDB::new(storage.into_super_arc(), Some(state_root));
        for block in retracted {
            for transaction in block.transactions().iter().rev() {
                let sender = transaction.sender();
                let sequence_number = match statedb.get_account_resource(sender) {
                    Ok(op_resource) => {
                        op_resource.map(|resource| resource.sequence_number()).unwrap_or_default()
                    }
                    Err(e) => bail!("Get account {} resource from statedb error: {:?}, return 0 as sequence_number", sender, e),
                };
                if transaction.expiration_timestamp_secs() >= now
                    && transaction.sequence_number() >= sequence_number
                {
                    self.pool
                        .lock()
                        .ok()
                        .and_then(|mut pool| Some(pool.push_front(transaction.clone())))
                        .unwrap();
                }
            }
        }
        Ok(())
    }

    fn status(&self) -> TxPoolStatus {
        unimplemented!("not implemented yet")
    }

    fn find_txn(&self, _hash: &HashValue) -> Option<SignedUserTransaction> {
        None // force to get the transactio from peer's storage
    }

    fn txns_of_sender(
        &self,
        _sender: &AccountAddress,
        _max_len: Option<usize>,
    ) -> Vec<SignedUserTransaction> {
        todo!()
    }

    fn get_pending_with_state(
        &self,
        max_len: u64,
        _current_timestamp_secs: Option<u64>,
        state_root: HashValue,
    ) -> Vec<SignedUserTransaction> {
        let mut result = vec![];
        let storage = self.storage.clone();
        let statedb = ChainStateDB::new(storage.into_super_arc(), Some(state_root));
        let mut pool = match self.pool.lock() {
            Ok(pool) => pool,
            Err(e) => {
                error!(
                    "MockTxPoolService get_pending_with_state lock pool error: {:?}",
                    e
                );
                return result;
            }
        };
        let max = std::cmp::min(pool.len() as u64, max_len);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        for i in 0..max {
            let txn = match pool.get(i as usize).cloned() {
                Some(txn) => txn,
                None => break,
            };
            let sender = txn.sender();
            let sequence_number = match statedb.get_account_resource(sender) {
                Ok(op_resource) => {
                    op_resource.map(|resource| resource.sequence_number()).unwrap_or_default()
                }
                Err(e) => panic!("in get_pending_with_state, Get account {} resource from statedb error: {:?}, return 0 as sequence_number", sender, e),
            };
            if txn.expiration_timestamp_secs() >= now && txn.sequence_number() >= sequence_number {
                result.push(txn);
            } else {
                pool.remove(i as usize);
            }
        }

        result
    }

    fn next_sequence_number_with_state(
        &self,
        address: AccountAddress,
        state_root: HashValue,
    ) -> Option<u64> {
        let storage = self.storage.clone();
        let statedb = ChainStateDB::new(storage.into_super_arc(), Some(state_root));
        match statedb.get_account_resource(address) {
            Ok(op_resource) => {
                op_resource.map(|resource| resource.sequence_number())
            }
            Err(e) => panic!("in get_pending_with_state, Get account {} resource from statedb error: {:?}, return 0 as sequence_number", address, e),
        }
    }

    fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<AccountAddress>,
    ) -> Option<Vec<(AccountAddress, Option<u64>)>> {
        let result = addresses
            .into_iter()
            .map(|addr| (addr, self.next_sequence_number(addr)))
            .collect();
        Some(result)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[stest::test]
//     async fn test_txpool() {
//         let pool = MockTxPoolService::new();

//         pool.add_txns(vec![SignedUserTransaction::mock()])
//             .pop()
//             .unwrap()
//             .unwrap();
//         let txns = pool.get_pending_txns(None, None);
//         assert_eq!(1, txns.len())
//     }
// }
