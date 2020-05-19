// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    pool,
    pool::{
        Gas, PendingOrdering, PendingSettings, PoolTransaction, PrioritizationStrategy, Status,
        TxStatus, UnverifiedUserTransaction, VerifiedTransaction,
    },
    pool_client::{NonceCache, PoolClient},
};
use anyhow::Result;
use common_crypto::hash::{HashValue, PlainCryptoHash};
use futures_channel::mpsc;
use parking_lot::RwLock;
use starcoin_config::TxPoolConfig;
use starcoin_txpool_api::TxPoolSyncService;
use std::sync::Arc;
use storage::Store;
use types::{
    account_address::AccountAddress, block::BlockHeader, transaction,
    transaction::SignedUserTransaction,
};

#[derive(Clone, Debug)]
pub struct TxPoolService {
    inner: Inner,
}
impl TxPoolService {
    pub fn new(
        pool_config: TxPoolConfig,
        storage: Arc<dyn Store>,
        chain_header: BlockHeader,
    ) -> Self {
        let verifier_options = pool::VerifierOptions {
            minimal_gas_price: pool_config.minimal_gas_price,
            block_gas_limit: Gas::max_value(),
            tx_gas_limit: pool_config.tx_gas_limit,
            no_early_reject: false,
        };
        let queue = TxnQueue::new(
            tx_pool::Options {
                max_count: pool_config.max_count as usize,
                max_mem_usage: pool_config.max_mem_usage as usize,
                max_per_sender: pool_config.max_per_sender as usize,
            },
            verifier_options,
            PrioritizationStrategy::GasPriceOnly,
        );
        let queue = Arc::new(queue);
        let inner = Inner {
            queue,
            storage,
            chain_header: Arc::new(RwLock::new(chain_header)),
            sequence_number_cache: NonceCache::new(128),
        };

        Self { inner }
    }

    pub(crate) fn from_inner(inner: Inner) -> TxPoolService {
        Self { inner }
    }
    pub(crate) fn get_inner(&self) -> Inner {
        self.inner.clone()
    }
}

impl TxPoolSyncService for TxPoolService {
    fn add_txns(
        &self,
        txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>> {
        self.inner.import_txns(txns)
    }

    fn remove_txn(&self, txn_hash: HashValue, is_invalid: bool) -> Option<SignedUserTransaction> {
        self.inner
            .remove_txn(txn_hash, is_invalid)
            .map(|t| t.signed().clone())
    }

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(&self, max_len: Option<u64>) -> Vec<SignedUserTransaction> {
        let r = self.inner.get_pending(max_len.unwrap_or(u64::MAX));
        r.into_iter().map(|t| t.signed().clone()).collect()
    }

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, address: AccountAddress) -> Option<u64> {
        self.inner.next_sequence_number(address)
    }

    /// subscribe
    fn subscribe_txns(
        &self,
    ) -> mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>> {
        self.inner.subscribe_txns()
    }

    /// rollback
    fn rollback(
        &self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        self.inner.chain_new_block(enacted, retracted)
    }
}

pub(crate) type TxnQueue = pool::TransactionQueue;
#[derive(Clone)]
pub(crate) struct Inner {
    queue: Arc<TxnQueue>,
    chain_header: Arc<RwLock<BlockHeader>>,
    storage: Arc<dyn Store>,
    sequence_number_cache: NonceCache,
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

    pub(crate) fn get_chain_header(&self) -> BlockHeader {
        self.chain_header.read().clone()
    }

    pub(crate) fn cull(&self) {
        // NOTICE: as the new head block event is sepeated with chain_new_block event,
        // we need to remove invalid txn here.
        // In fact, it would be better if caller can make it into one.
        // In this situation, we don't need to reimport invalid txn on chain_new_block.
        self.queue.cull(self.get_pool_client())
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
    pub(crate) fn get_pending(&self, max_len: u64) -> Vec<Arc<VerifiedTransaction>> {
        let pending_settings = PendingSettings {
            block_number: u64::max_value(),
            current_timestamp: u64::max_value(),
            nonce_cap: None,
            max_len: max_len as usize,
            ordering: PendingOrdering::Priority,
        };
        self.queue.pending(self.get_pool_client(), pending_settings)
    }
    pub(crate) fn next_sequence_number(&self, address: AccountAddress) -> Option<u64> {
        self.queue
            .next_sequence_number(self.get_pool_client(), &address)
    }

    pub(crate) fn subscribe_txns(
        &self,
    ) -> mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>> {
        let (tx, rx) = mpsc::unbounded();
        self.queue.add_full_listener(tx);
        rx
    }
    pub(crate) fn subscribe_pending_txns(&self) -> mpsc::UnboundedReceiver<Arc<Vec<HashValue>>> {
        let (tx, rx) = mpsc::unbounded();
        self.queue.add_pending_listener(tx);
        rx
    }

    pub(crate) fn chain_new_block(
        &self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        info!(
            "receive chain_new_block msg, enacted: {:?}, retracted: {:?}",
            enacted
                .iter()
                .map(|t| (t.sender(), t.sequence_number()))
                .collect::<Vec<_>>(),
            retracted
                .iter()
                .map(|t| (t.sender(), t.sequence_number()))
                .collect::<Vec<_>>(),
        );

        let hashes: Vec<_> = enacted.iter().map(|t| t.crypto_hash()).collect();
        let _ = self.queue.remove(hashes.iter(), false);

        let txns = retracted
            .into_iter()
            .map(|t| PoolTransaction::Retracted(UnverifiedUserTransaction::from(t)));
        let _ = self.queue.import(self.get_pool_client(), txns);
        // ignore import result
        // for r in import_result {
        //     r?;
        // }
        // self.queue.cull(client);
        Ok(())
    }

    fn get_pool_client(&self) -> PoolClient {
        PoolClient::new(
            self.chain_header.read().clone(),
            self.storage.clone(),
            self.sequence_number_cache.clone(),
        )
    }
}
