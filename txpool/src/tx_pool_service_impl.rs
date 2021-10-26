// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    pool,
    pool::{
        PendingOrdering, PendingSettings, PoolTransaction, PrioritizationStrategy, Status,
        TxStatus, UnverifiedUserTransaction, VerifiedTransaction,
    },
    pool_client::{NonceCache, PoolClient},
};

use crate::metrics::TxPoolMetrics;
use crate::pool::{Client, TransactionQueue};
use anyhow::Result;
use crypto::hash::HashValue;
use futures_channel::mpsc;
use parking_lot::RwLock;
use starcoin_config::NodeConfig;
use starcoin_executor::VMMetrics;
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService};
use std::sync::Arc;
use storage::Store;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader},
    transaction,
    transaction::SignedUserTransaction,
};

#[derive(Clone, Debug)]
pub struct TxPoolService {
    inner: Inner,
}
impl TxPoolService {
    pub fn new(
        node_config: Arc<NodeConfig>,
        storage: Arc<dyn Store>,
        chain_header: BlockHeader,
        vm_metrics: Option<VMMetrics>,
    ) -> Self {
        let metrics = node_config
            .metrics
            .registry()
            .and_then(|registry| TxPoolMetrics::register(registry).ok());

        let pool_config = &node_config.tx_pool;
        let verifier_options = pool::VerifierOptions {
            no_early_reject: false,
            min_gas_price: node_config.tx_pool.min_gas_price(),
        };
        let queue = TxnQueue::new(
            tx_pool::Options {
                max_count: pool_config.max_count() as usize,
                max_mem_usage: pool_config.max_mem_usage() as usize,
                max_per_sender: pool_config.max_per_sender() as usize,
            },
            verifier_options,
            PrioritizationStrategy::GasPriceOnly,
        );
        let queue = Arc::new(queue);
        let inner = Inner {
            node_config,
            queue,
            storage,
            chain_header: Arc::new(RwLock::new(chain_header)),
            sequence_number_cache: NonceCache::new(128),
            metrics,
            vm_metrics,
        };

        Self { inner }
    }

    pub fn get_store(&self) -> Arc<dyn Store> {
        self.inner.storage.clone()
    }

    pub(crate) fn from_inner(inner: Inner) -> TxPoolService {
        Self { inner }
    }
    pub(crate) fn get_inner(&self) -> Inner {
        self.inner.clone()
    }

    pub fn verify_transaction(
        &self,
        tx: SignedUserTransaction,
    ) -> Result<transaction::SignatureCheckedTransaction, transaction::TransactionError> {
        self.get_inner()
            .get_pool_client()
            .verify_transaction(tx.into())
    }
}

impl TxPoolSyncService for TxPoolService {
    fn add_txns(
        &self,
        txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>> {
        // _timer will observe_duration when it's dropped.
        // We don't need to call it explicitly.
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["add_txns"])
                .start_timer()
        });
        self.inner.import_txns(txns)
    }

    fn remove_txn(&self, txn_hash: HashValue, is_invalid: bool) -> Option<SignedUserTransaction> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["remove_txn"])
                .start_timer()
        });
        self.inner
            .remove_txn(txn_hash, is_invalid)
            .map(|t| t.signed().clone())
    }

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(
        &self,
        max_len: Option<u64>,
        current_timestamp_secs: Option<u64>,
    ) -> Vec<SignedUserTransaction> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["get_pending_txns"])
                .start_timer()
        });
        let current_timestamp_secs = current_timestamp_secs
            .unwrap_or_else(|| self.inner.node_config.net().time_service().now_secs());
        let r = self
            .inner
            .get_pending(max_len.unwrap_or(u64::MAX), current_timestamp_secs);
        r.into_iter().map(|t| t.signed().clone()).collect()
    }

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, address: AccountAddress) -> Option<u64> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["next_sequence_number"])
                .start_timer()
        });
        self.inner.next_sequence_number(address)
    }

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<Arc<[(HashValue, transaction::TxStatus)]>> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["subscribe_txns"])
                .start_timer()
        });
        self.inner.subscribe_txns()
    }

    fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["subscribe_pending_txns"])
                .start_timer()
        });
        self.inner.subscribe_pending_txns()
    }

    /// rollback
    fn chain_new_block(&self, enacted: Vec<Block>, retracted: Vec<Block>) -> Result<()> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["rollback"])
                .start_timer()
        });
        self.inner.chain_new_block(enacted, retracted);
        Ok(())
    }

    fn status(&self) -> TxPoolStatus {
        self.inner.queue.status().into()
    }

    fn find_txn(&self, hash: &HashValue) -> Option<SignedUserTransaction> {
        self.inner
            .queue
            .find(hash)
            .map(move |txn| txn.signed().clone())
    }
    fn txns_of_sender(
        &self,
        sender: &AccountAddress,
        max_len: Option<usize>,
    ) -> Vec<SignedUserTransaction> {
        self.inner
            .queue
            .txns_of_sender(sender, max_len.unwrap_or(usize::max_value()))
            .into_iter()
            .map(|t| t.signed().clone())
            .collect()
    }
}

pub(crate) type TxnQueue = TransactionQueue;
#[derive(Clone)]
pub(crate) struct Inner {
    pub(crate) node_config: Arc<NodeConfig>,
    queue: Arc<TxnQueue>,
    chain_header: Arc<RwLock<BlockHeader>>,
    storage: Arc<dyn Store>,
    sequence_number_cache: NonceCache,
    pub(crate) metrics: Option<TxPoolMetrics>,
    vm_metrics: Option<VMMetrics>,
}
impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "queue: {:?}, chain header: {:?}",
            &self.queue, &self.chain_header,
        )
    }
}

impl Inner {
    pub(crate) fn queue(&self) -> Arc<TxnQueue> {
        self.queue.clone()
    }
    pub(crate) fn pool_status(&self) -> Status {
        self.queue.status()
    }

    pub(crate) fn notify_new_chain_header(&self, header: BlockHeader) {
        *self.chain_header.write() = header;
        self.sequence_number_cache.clear();
    }

    pub(crate) fn get_chain_reader(&self) -> ChainStateDB {
        ChainStateDB::new(
            self.storage.clone().into_super_arc(),
            Some(self.get_chain_header().state_root()),
        )
    }
    pub(crate) fn get_chain_header(&self) -> BlockHeader {
        self.chain_header.read().clone()
    }

    pub(crate) fn cull(&self) {
        // NOTICE: as the new head block event is sepeated with chain_new_block event,
        // we need to remove invalid txn here.
        // In fact, it would be better if caller can make it into one.
        // In this situation, we don't need to reimport invalid txn on chain_new_block.
        let now_seconds = self.chain_header.read().timestamp() / 1000;
        self.queue.cull(self.get_pool_client(), now_seconds)
    }

    pub(crate) fn import_txns(
        &self,
        txns: Vec<transaction::SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>> {
        let txns = txns
            .into_iter()
            .map(|t| PoolTransaction::Unverified(UnverifiedUserTransaction::from(t)));
        self.queue.import(self.get_pool_client(), txns)
    }
    pub(crate) fn remove_txn(
        &self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Option<Arc<pool::VerifiedTransaction>> {
        let mut removed = self.queue.remove(vec![&txn_hash], is_invalid);
        removed
            .pop()
            .expect("remove should return one result per hash")
    }
    pub(crate) fn get_pending(
        &self,
        max_len: u64,
        current_timestamp_secs: u64,
    ) -> Vec<Arc<VerifiedTransaction>> {
        let pending_settings = PendingSettings {
            block_number: u64::max_value(),
            current_timestamp: current_timestamp_secs,
            max_len: max_len as usize,
            ordering: PendingOrdering::Priority,
        };
        self.queue.inner_status(
            self.get_pool_client(),
            u64::max_value(),
            current_timestamp_secs,
        );
        self.queue.pending(self.get_pool_client(), pending_settings)
    }
    pub(crate) fn next_sequence_number(&self, address: AccountAddress) -> Option<u64> {
        self.queue
            .next_sequence_number(self.get_pool_client(), &address)
    }

    pub(crate) fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<Arc<[(HashValue, TxStatus)]>> {
        let (tx, rx) = mpsc::unbounded();
        self.queue.add_full_listener(tx);
        rx
    }
    pub(crate) fn subscribe_pending_txns(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>> {
        let (tx, rx) = mpsc::unbounded();
        self.queue.add_pending_listener(tx);
        rx
    }

    pub(crate) fn chain_new_block(&self, enacted: Vec<Block>, retracted: Vec<Block>) {
        debug!(
            "receive chain_new_block msg, enacted: {:?}, retracted: {:?}",
            enacted
                .iter()
                .map(|b| b.header().number())
                .collect::<Vec<_>>(),
            retracted
                .iter()
                .map(|b| b.header().number())
                .collect::<Vec<_>>()
        );

        // new head block, update chain header
        if let Some(block) = enacted.last() {
            self.notify_new_chain_header(block.header().clone());
        }

        // remove outdated txns.
        self.cull();

        // import retracted txns.
        let txns = retracted
            .into_iter()
            .flat_map(|b| {
                let txns: Vec<SignedUserTransaction> = b.into_inner().1.into();
                txns.into_iter()
            })
            .map(|t| PoolTransaction::Retracted(UnverifiedUserTransaction::from(t)));
        let results = self.queue.import(self.get_pool_client(), txns);
        for result in results {
            if let Err(err) = result {
                debug!("retracted transaction fail: {}", err);
            }
        }
    }

    fn get_pool_client(&self) -> PoolClient {
        PoolClient::new(
            self.chain_header.read().clone(),
            self.storage.clone(),
            self.sequence_number_cache.clone(),
            self.vm_metrics.clone(),
        )
    }
}
