// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TxPoolAsyncService;
use anyhow::Result;
use crypto::hash::HashValue;
use futures_channel::mpsc;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};
use types::transaction;
use types::transaction::SignedUserTransaction;
#[derive(Clone)]
pub struct MockTxPoolService {
    pool: Arc<Mutex<Vec<SignedUserTransaction>>>,
}

impl MockTxPoolService {
    pub fn new() -> Self {
        Self::new_with_txns(vec![])
    }

    pub fn new_with_txns(txns: Vec<SignedUserTransaction>) -> Self {
        MockTxPoolService {
            pool: Arc::new(Mutex::new(txns)),
        }
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
    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>> {
        match max_len {
            Some(max) => Ok(self
                .pool
                .lock()
                .unwrap()
                .iter()
                .take(max as usize)
                .map(|c| c.clone())
                .collect::<Vec<_>>()),
            None => Ok(self.pool.lock().unwrap().clone()),
        }
    }

    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>>> {
        unimplemented!()
    }

    async fn chain_new_blocks(
        self,
        enacted: Vec<HashValue>,
        retracted: Vec<HashValue>,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
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
