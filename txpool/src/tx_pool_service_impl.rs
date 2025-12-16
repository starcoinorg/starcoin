// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    pool,
    pool::{
        PendingOrdering, PendingSettings, PoolTransaction, PrioritizationStrategy, Status,
        UnverifiedUserTransaction, VerifiedTransaction,
    },
    pool_client::{NonceCache, PoolClient},
    verifier_pool::VerifierPool,
};

use crate::metrics::TxPoolMetrics;
use crate::pool::{Client, TransactionQueue};
use anyhow::Result;
use futures_channel::mpsc;
use parking_lot::RwLock;
use starcoin_config::NodeConfig;
use starcoin_crypto::hash::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::info;
use starcoin_storage::Store;
use starcoin_storage::Store2;
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService, TxnStatusFullEvent};
use starcoin_types::multi_transaction::{
    APIInterruptedError, MultiAccountAddress, MultiSignatureCheckedTransaction,
    MultiSignedUserTransaction, MultiTransactionError,
};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader},
};
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TxPoolService {
    pub inner: Inner,
}
impl TxPoolService {
    pub fn new(
        node_config: Arc<NodeConfig>,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        chain_header: BlockHeader,
        vm_metrics: Option<VMMetrics>,
    ) -> Self {
        let metrics = node_config
            .metrics
            .registry()
            .and_then(|registry| TxPoolMetrics::register(registry).ok());

        let pool_config = &node_config.tx_pool;
        let verifier_pool = if pool_config.verifier_pool_enabled() {
            Some(Arc::new(VerifierPool::new(
                pool_config.verifier_pool_size(),
                storage2.clone(),
                vm_metrics.clone(),
            )))
        } else {
            None
        };
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
            pool_config.max_vm1_txn_count(),
            pool_config.max_vm1_rejections_per_peer(),
            pool_config.vm1_peer_blacklist_duration_secs(),
        );
        let queue = Arc::new(queue);
        let inner = Inner {
            node_config,
            queue,
            storage,
            storage2,
            chain_header: Arc::new(RwLock::new(chain_header)),
            sequence_number_cache: NonceCache::new(128),
            metrics,
            vm_metrics,
            verifier_pool,
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
        tx: MultiSignedUserTransaction,
    ) -> Result<MultiSignatureCheckedTransaction, MultiTransactionError> {
        self.get_inner()
            .get_pool_client()
            .map_err(|e| MultiTransactionError::APIInterrupted(APIInterruptedError(e.to_string())))?
            .verify_transaction(tx.into())
    }
}

impl TxPoolSyncService for TxPoolService {
    fn add_txns_multi_signed(
        &self,
        txns: Vec<MultiSignedUserTransaction>,
        bypass_vm1_limit: bool,
        peer_id: Option<String>,
    ) -> Result<Vec<Result<(), MultiTransactionError>>> {
        // _timer will observe_duration when it's dropped.
        // We don't need to call it explicitly.
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["add_txns"])
                .start_timer()
        });
        self.inner.import_txns(txns, bypass_vm1_limit, peer_id)
    }

    fn remove_txn(
        &self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Option<MultiSignedUserTransaction> {
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
    ) -> Result<Vec<MultiSignedUserTransaction>> {
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
            .get_pending(max_len.unwrap_or(u64::MAX), current_timestamp_secs)?;
        Ok(r.into_iter().map(|t| t.signed().clone()).collect())
    }

    fn get_pending_with_state(
        &self,
        max_len: u64,
        current_timestamp_secs: Option<u64>,
        state_root1: HashValue,
        state_root2: HashValue,
    ) -> Result<Vec<MultiSignedUserTransaction>> {
        let _timer: Option<starcoin_metrics::HistogramTimer> =
            self.inner.metrics.as_ref().map(|metrics| {
                metrics
                    .txpool_service_time
                    .with_label_values(&["get_pending_with_pool_client"])
                    .start_timer()
            });
        let current_timestamp_secs = current_timestamp_secs
            .unwrap_or_else(|| self.inner.node_config.net().time_service().now_secs());
        let pool_client = PoolClient::new(
            state_root1,
            state_root2,
            self.inner.storage.clone(),
            self.inner.storage2.clone(),
            NonceCache::new(0),
            self.inner.vm_metrics.clone(),
            self.inner.verifier_pool.clone(),
        );
        let r =
            self.inner
                .get_pending_with_pool_client(max_len, current_timestamp_secs, pool_client);
        Ok(r.into_iter().map(|t| t.signed().clone()).collect())
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
        self.inner
            .next_sequence_number(MultiAccountAddress::VM1(address))
    }

    fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<AccountAddress>,
    ) -> Option<Vec<(AccountAddress, Option<u64>)>> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["next_sequence_number"])
                .start_timer()
        });
        self.inner
            .next_sequence_number_in_batch(
                addresses
                    .into_iter()
                    .map(MultiAccountAddress::VM1)
                    .collect(),
            )
            .map(|results| {
                results
                    .into_iter()
                    .map(|(address, seq)| {
                        (
                            match address {
                                MultiAccountAddress::VM1(account_address) => account_address,
                                MultiAccountAddress::VM2(_account_address) => panic!(
                                    "unexpected VM2 account address in next_sequence_number2_in_batch"
                                ),
                            },
                            seq,
                        )
                    })
                    .collect()
            })
    }

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent> {
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

    fn find_txn(&self, hash: &HashValue) -> Option<MultiSignedUserTransaction> {
        self.inner
            .queue
            .find(hash)
            .map(move |txn| txn.signed().clone())
    }
    fn txns_of_sender(
        &self,
        sender: &MultiAccountAddress,
        max_len: Option<usize>,
    ) -> Vec<MultiSignedUserTransaction> {
        self.inner
            .queue
            .txns_of_sender(sender, max_len.unwrap_or(usize::MAX))
            .into_iter()
            .map(|t| t.signed().clone())
            .collect()
    }

    fn next_sequence_number2(&self, address: AccountAddress2) -> Option<u64> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["next_sequence_number2"])
                .start_timer()
        });
        self.inner
            .next_sequence_number(MultiAccountAddress::VM2(address))
    }

    fn next_sequence_number2_in_batch(
        &self,
        addresses: Vec<AccountAddress2>,
    ) -> Option<Vec<(AccountAddress2, Option<u64>)>> {
        let _timer = self.inner.metrics.as_ref().map(|metrics| {
            metrics
                .txpool_service_time
                .with_label_values(&["next_sequence_number"])
                .start_timer()
        });
        self.inner
            .next_sequence_number_in_batch(
                addresses
                    .into_iter()
                    .map(MultiAccountAddress::VM2)
                    .collect(),
            )
            .map(|results| {
                results
                    .into_iter()
                    .map(|(address, seq)| {
                        (
                            match address {
                                MultiAccountAddress::VM1(_account_address) => panic!(
                                    "unexpected account address in next_sequence_number2_in_batch"
                                ),
                                MultiAccountAddress::VM2(account_address) => account_address,
                            },
                            seq,
                        )
                    })
                    .collect()
            })
    }
}

pub(crate) type TxnQueue = TransactionQueue;
#[derive(Clone)]
pub struct Inner {
    pub(crate) node_config: Arc<NodeConfig>,
    queue: Arc<TxnQueue>,
    chain_header: Arc<RwLock<BlockHeader>>,
    storage: Arc<dyn Store>,
    storage2: Arc<dyn Store2>,
    sequence_number_cache: NonceCache,
    pub(crate) metrics: Option<TxPoolMetrics>,
    vm_metrics: Option<VMMetrics>,
    verifier_pool: Option<Arc<VerifierPool>>,
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
    pub fn queue(&self) -> Arc<TxnQueue> {
        self.queue.clone()
    }
    pub(crate) fn pool_status(&self) -> Status {
        self.queue.status()
    }

    pub(crate) fn notify_new_chain_header(&self, header: BlockHeader) {
        *self.chain_header.write() = header;
        self.sequence_number_cache.clear();
        if let Some(pool) = &self.verifier_pool {
            pool.invalidate_all();
        }
    }

    pub(crate) fn get_chain_reader(&self) -> Result<ChainStateDB> {
        let multi_state = self
            .storage
            .get_vm_multi_state(self.chain_header.read().id())?;
        Ok(ChainStateDB::new(
            self.storage2.clone().into_super_arc(),
            Some(multi_state.state_root2()),
        ))
    }

    pub(crate) fn cull(&self) -> Result<()> {
        // NOTICE: as the new head block event is repeated with chain_new_block event,
        // we need to remove invalid txn here.
        // In fact, it would be better if caller can make it into one.
        // In this situation, we don't need to reimport invalid txn on chain_new_block.
        let now_seconds = self.chain_header.read().timestamp() / 1000;
        self.queue.cull(self.get_pool_client()?, now_seconds);
        Ok(())
    }

    pub fn import_txns(
        &self,
        txns: Vec<MultiSignedUserTransaction>,
        bypass_vm1_limit: bool,
        peer_id: Option<String>,
    ) -> Result<Vec<Result<(), MultiTransactionError>>> {
        let import_time = std::time::Instant::now();
        let txn_count = txns.len();
        let txn_hashes: Vec<_> = txns.iter().map(|t| t.id()).collect();
        info!(
            "[jacktest] txpool import_txns start: count={}, first_hash={:?}",
            txn_count,
            txn_hashes.first()
        );
        let txns = txns
            .into_iter()
            .map(|t| PoolTransaction::Unverified(UnverifiedUserTransaction::from(t)));
        let result = self
            .queue
            .import(self.get_pool_client()?, txns, bypass_vm1_limit, peer_id);
        info!(
            "[jacktest] txpool import_txns done: count={}, elapsed_ms={}, first_hash={:?}",
            txn_count,
            import_time.elapsed().as_millis(),
            txn_hashes.first()
        );
        Ok(result)
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
    pub fn get_pending(
        &self,
        max_len: u64,
        current_timestamp_secs: u64,
    ) -> Result<Vec<Arc<VerifiedTransaction>>> {
        // let pending_settings = PendingSettings {
        //     block_number: u64::MAX,
        //     current_timestamp: current_timestamp_secs,
        //     max_len: max_len as usize,
        //     ordering: PendingOrdering::Priority,
        // };
        // self.queue
        //     .inner_status(self.get_pool_client(), u64::MAX, current_timestamp_secs);
        // self.queue.pending(self.get_pool_client(), pending_settings)
        Ok(self.get_pending_with_pool_client(
            max_len,
            current_timestamp_secs,
            self.get_pool_client()?,
        ))
    }

    pub fn get_pending_with_pool_client(
        &self,
        max_len: u64,
        current_timestamp_secs: u64,
        pool_client: PoolClient,
    ) -> Vec<Arc<VerifiedTransaction>> {
        let start_time = std::time::Instant::now();
        let pool_status = self.queue.status();
        info!(
            "[jacktest] get_pending start: max_len={}, pool_status={:?}",
            max_len, pool_status
        );
        let pending_settings = PendingSettings {
            block_number: u64::MAX,
            current_timestamp: current_timestamp_secs,
            max_len: max_len as usize,
            ordering: PendingOrdering::Priority,
        };
        // why here calls inner_status???
        // self.queue
        //     .inner_status(self.get_pool_client(), u64::MAX, current_timestamp_secs);
        let result = self.queue.pending(pool_client, pending_settings);
        info!(
            "[jacktest] get_pending done: returned={}, elapsed_ms={}, pool_txn_count={}",
            result.len(),
            start_time.elapsed().as_millis(),
            pool_status.status.transaction_count
        );
        result
    }

    pub fn try_read(&self) -> Option<parking_lot::RwLockReadGuard<crate::Pool>> {
        self.queue.try_read()
    }

    pub(crate) fn next_sequence_number(&self, address: MultiAccountAddress) -> Option<u64> {
        let client = match self.get_pool_client() {
            Ok(client) => client,
            Err(e) => {
                error!("failed to get pool client in next_sequence_number: {}", e);
                return None;
            }
        };
        self.queue.next_sequence_number(client, &address)
    }

    pub(crate) fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<MultiAccountAddress>,
    ) -> Option<Vec<(MultiAccountAddress, Option<u64>)>> {
        let (state_root1, state_root2) = match self
            .storage
            .get_vm_multi_state(self.chain_header.read().id())
        {
            Ok(multi_state) => (multi_state.state_root1(), multi_state.state_root2()),
            Err(e) => {
                error!(
                    "failed to get vm multi state in next_sequence_number_in_batch: {}",
                    e
                );
                return None;
            }
        };
        let pool_client = PoolClient::new(
            state_root1,
            state_root2,
            self.storage.clone(),
            self.storage2.clone(),
            NonceCache::new(0),
            self.vm_metrics.clone(),
            self.verifier_pool.clone(),
        );
        self.queue
            .next_sequence_number_in_batch(pool_client, addresses)
    }

    pub(crate) fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent> {
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
        if let Err(e) = self.cull() {
            error!("failed to cull in chain_new_block: {}", e);
            return;
        }

        // import retracted txns.
        let txns = retracted
            .into_iter()
            .flat_map(|b| {
                let txns: Vec<MultiSignedUserTransaction> = b.into_inner().1.into();
                txns.into_iter()
            })
            .map(|t| PoolTransaction::Retracted(UnverifiedUserTransaction::from(t)));
        let client = match self.get_pool_client() {
            Ok(client) => client,
            Err(e) => {
                error!("failed to get pool client in chain_new_block: {}", e);
                return;
            }
        };
        let results = self.queue.import(client, txns, true, None);
        for result in results {
            if let Err(err) = result {
                debug!("retracted transaction fail: {}", err);
            }
        }
    }

    pub fn get_pool_client(&self) -> Result<PoolClient> {
        let state = self
            .storage
            .get_vm_multi_state(self.chain_header.read().id())?;
        let (state_root1, state_root2) = (state.state_root1(), state.state_root2());
        Ok(PoolClient::new(
            state_root1,
            state_root2,
            self.storage.clone(),
            self.storage2.clone(),
            self.sequence_number_cache.clone(),
            self.vm_metrics.clone(),
            self.verifier_pool.clone(),
        ))
    }
}
