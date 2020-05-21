// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::hash::HashValue;
use futures_channel::mpsc;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};
use types::account_address::AccountAddress;
use types::transaction;
use types::transaction::SignedUserTransaction;

#[derive(Clone, Default)]
pub struct MockTxPoolService {
    pool: Arc<Mutex<Vec<SignedUserTransaction>>>,
}

impl MockTxPoolService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_txns(txns: Vec<SignedUserTransaction>) -> Self {
        MockTxPoolService {
            pool: Arc::new(Mutex::new(txns)),
        }
    }
}

impl TxPoolSyncService for MockTxPoolService {
    fn add_txns(
        &self,
        mut txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>> {
        let len = txns.len();
        self.pool.lock().unwrap().append(&mut txns);
        let mut results = vec![];
        results.resize_with(len, || Ok(()));
        results
    }

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    fn remove_txn(&self, txn_hash: HashValue, is_invalid: bool) -> Option<SignedUserTransaction> {
        unimplemented!()
    }

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(&self, max_len: Option<u64>) -> Vec<SignedUserTransaction> {
        match max_len {
            Some(max) => self
                .pool
                .lock()
                .unwrap()
                .iter()
                .take(max as usize)
                .cloned()
                .collect::<Vec<_>>(),
            None => self.pool.lock().unwrap().clone(),
        }
    }

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, address: AccountAddress) -> Option<u64> {
        todo!()
    }

    /// subscribe
    fn subscribe_txns(
        &self,
    ) -> mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>> {
        todo!()
    }

    /// rollback
    fn rollback(
        &self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        todo!()
    }
}

#[async_trait::async_trait]
impl TxPoolAsyncService for MockTxPoolService {
    async fn add(self, txn: SignedUserTransaction) -> Result<bool> {
        self.pool.lock().unwrap().push(txn);
        //TODO check txn is exist.
        Ok(true)
    }
    async fn add_txns(
        self,
        mut txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>> {
        let len = txns.len();
        self.pool.lock().unwrap().append(&mut txns);
        let mut results = vec![];
        results.resize_with(len, || Ok(()));
        Ok(results)
    }
    async fn remove_txn(
        self,
        _txn_hash: HashValue,
        _is_invalid: bool,
    ) -> Result<Option<SignedUserTransaction>> {
        unimplemented!()
    }

    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>> {
        match max_len {
            Some(max) => Ok(self
                .pool
                .lock()
                .unwrap()
                .iter()
                .take(max as usize)
                .cloned()
                .collect::<Vec<_>>()),
            None => Ok(self.pool.lock().unwrap().clone()),
        }
    }
    async fn next_sequence_number(self, _address: AccountAddress) -> Result<Option<u64>> {
        todo!()
    }

    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>>> {
        unimplemented!()
    }

    async fn rollback(
        self,
        _enacted: Vec<SignedUserTransaction>,
        _retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_txpool() {
        let pool = MockTxPoolService::new();

        pool.clone()
            .add(SignedUserTransaction::mock())
            .await
            .unwrap();
        let txns = pool.get_pending_txns(None).await.unwrap();
        assert_eq!(1, txns.len())
    }
}
