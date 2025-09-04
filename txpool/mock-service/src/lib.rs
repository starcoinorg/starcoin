// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use futures_channel::mpsc;
use starcoin_crypto::hash::HashValue;
use starcoin_state_api::{AccountStateReader, StateReaderExt};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{storage, Store};
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService, TxnStatusFullEvent};
use starcoin_types::{
    account_address::AccountAddress, block::Block, transaction, transaction::SignedUserTransaction,
};
use std::{
    collections::VecDeque,
    iter::Iterator,
    sync::{Arc, Mutex},
};

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
}

impl TxPoolSyncService for MockTxPoolService {
    fn add_txns(
        &self,
        txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>> {
        let len = txns.len();
        self.pool.lock().unwrap().append(&mut txns.into());
        let mut results = vec![];
        results.resize_with(len, || Ok(()));
        results
    }

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    fn remove_txn(&self, txn_hash: HashValue, _is_invalid: bool) -> Option<SignedUserTransaction> {
        self.pool
            .lock()
            .unwrap()
            .iter()
            .position(|t| t.id() == txn_hash)
            .map(|i| self.pool.lock().unwrap().remove(i).unwrap())
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
                .unwrap()
                .drain(0..max as usize)
                .collect::<Vec<_>>(),
            None => self.pool.lock().unwrap().drain(..).collect::<Vec<_>>(),
        }
    }

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, _address: AccountAddress) -> Option<u64> {
        None
    }

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent> {
        todo!()
    }
    fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>> {
        todo!()
    }

    fn chain_new_block(&self, enacted: Vec<Block>, retracted: Vec<Block>) -> Result<()> {
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
                if transaction.sequence_number() > sequence_number {
                    self.pool.lock().unwrap().push_front(transaction.clone());
                }
            }
        }
        Ok(())
    }

    fn status(&self) -> TxPoolStatus {
        unimplemented!()
    }

    fn find_txn(&self, _hash: &HashValue) -> Option<SignedUserTransaction> {
        unimplemented!()
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
        let max = std::cmp::max(self.pool.lock().unwrap().len() as u64, max_len);
        for i in 0..max {
            let txn = self.pool.lock().unwrap().get(i as usize).cloned().unwrap();
            let sender = txn.sender();
            let sequence_number = match statedb.get_account_resource(sender) {
                Ok(op_resource) => {
                    op_resource.map(|resource| resource.sequence_number()).unwrap_or_default()
                }
                Err(e) => panic!("in get_pending_with_state, Get account {} resource from statedb error: {:?}, return 0 as sequence_number", sender, e),
            };
            if txn.sequence_number() > sequence_number {
                result.push(txn);
            } else {
                self.pool.lock().unwrap().remove(i as usize);
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
