// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_channel::mpsc;
use starcoin_crypto::hash::HashValue;
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService, TxnStatusFullEvent};
use starcoin_types::multi_transaction::{
    MultiAccountAddress, MultiSignedUserTransaction, MultiTransactionError,
};
use starcoin_types::{account_address::AccountAddress, block::Block};
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use std::{
    iter::Iterator,
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub struct MockTxPoolService {
    pool: Arc<Mutex<Vec<MultiSignedUserTransaction>>>,
}

impl MockTxPoolService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_txns(txns: Vec<MultiSignedUserTransaction>) -> Self {
        MockTxPoolService {
            pool: Arc::new(Mutex::new(txns)),
        }
    }
}

impl TxPoolSyncService for MockTxPoolService {
    fn add_txns_multi_signed(
        &self,
        mut txns: Vec<MultiSignedUserTransaction>,
        _bypass_vm1_limit: u64,
        _peer_id: Option<String>,
    ) -> Vec<Result<(), MultiTransactionError>> {
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
    fn remove_txn(
        &self,
        _txn_hash: HashValue,
        _is_invalid: bool,
    ) -> Option<MultiSignedUserTransaction> {
        unimplemented!()
    }

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(
        &self,
        max_len: Option<u64>,
        _now: Option<u64>,
    ) -> Vec<MultiSignedUserTransaction> {
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
    fn next_sequence_number(&self, _address: AccountAddress) -> Option<u64> {
        todo!()
    }

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent> {
        todo!()
    }
    fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>> {
        todo!()
    }
    fn chain_new_block(&self, _enacted: Vec<Block>, _retracted: Vec<Block>) -> Result<()> {
        Ok(())
    }

    fn status(&self) -> TxPoolStatus {
        unimplemented!()
    }

    fn find_txn(&self, _hash: &HashValue) -> Option<MultiSignedUserTransaction> {
        unimplemented!()
    }

    fn txns_of_sender(
        &self,
        _sender: &MultiAccountAddress,
        _max_len: Option<usize>,
    ) -> Vec<MultiSignedUserTransaction> {
        unimplemented!("no need implemented for MockTxPoolService")
    }

    fn next_sequence_number2(&self, _address: AccountAddress2) -> Option<u64> {
        unimplemented!("no need implemented for MockTxPoolService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_types::transaction::SignedUserTransaction;

    #[stest::test]
    async fn test_txpool() {
        let pool = MockTxPoolService::new();

        pool.add_txns(vec![SignedUserTransaction::mock()])
            .pop()
            .unwrap()
            .unwrap();
        let txns = pool.get_pending_txns(None, None);
        assert_eq!(1, txns.len())
    }
}
