// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, sync::Arc};

use super::{TxStatus, VerifiedTransaction as Transaction};
use crypto::hash::HashValue as H256;
use futures_channel::mpsc;
use transaction_pool as tx_pool;
use tx_pool::VerifiedTransaction;

/// Transaction pool logger.
#[derive(Default, Debug)]
pub struct Logger;

impl tx_pool::Listener<Transaction> for Logger {
    fn added(&mut self, tx: &Arc<Transaction>, old: Option<&Arc<Transaction>>) {
        debug!(
            target: "txqueue",
            "Txn added. [{hash:?}] Sender: {sender}, nonce: {nonce}, gasPrice: {gas_price}, gas: {gas}, expiration_timestamp_secs: {expiration_timestamp_secs}, dataLen: {data:?}))",
            hash = tx.hash(),
            sender = tx.sender(),
            nonce = tx.signed().sequence_number(),
            gas_price = tx.signed().gas_unit_price(),
            gas = tx.signed().max_gas_amount(),
            expiration_timestamp_secs = tx.signed().expiration_timestamp_secs(),
            data = tx.signed().payload(),
        );
        if let Some(old) = old {
            debug!(target: "txqueue", "[{:?}] Dropped. Replaced by [{:?}]", old.hash(), tx.hash());
        }
    }

    fn rejected<H: fmt::Debug + fmt::LowerHex>(
        &mut self,
        tx: &Arc<Transaction>,
        reason: &tx_pool::Error<H>,
    ) {
        debug!(target: "txqueue", "[{hash:?}] Rejected. {reason}.",  hash = tx.hash(), reason = reason);
    }

    fn dropped(&mut self, tx: &Arc<Transaction>, new: Option<&Transaction>) {
        match new {
            Some(new) => {
                debug!(target: "txqueue", "[{:?}] Pushed out by [{:?}]", tx.hash(), new.hash())
            }
            None => debug!(target: "txqueue", "[{:?}] Dropped.", tx.hash()),
        }
    }

    fn invalid(&mut self, tx: &Arc<Transaction>) {
        debug!(target: "txqueue", "[{:?}] Marked as invalid by executor.", tx.hash());
    }

    fn canceled(&mut self, tx: &Arc<Transaction>) {
        debug!(target: "txqueue", "[{:?}] Canceled by the user.", tx.hash());
    }

    fn culled(&mut self, tx: &Arc<Transaction>) {
        debug!(target: "txqueue", "[{:?}] Culled or mined.", tx.hash());
    }
}

/// Transactions pool notifier
#[derive(Default)]
pub struct TransactionsPoolNotifier {
    full_listeners: Vec<mpsc::UnboundedSender<Arc<[(H256, TxStatus)]>>>,
    pending_listeners: Vec<mpsc::UnboundedSender<Arc<[H256]>>>,
    tx_statuses: Vec<(H256, TxStatus)>,
}

impl TransactionsPoolNotifier {
    /// Add new full listener to receive notifications.
    pub fn add_full_listener(&mut self, f: mpsc::UnboundedSender<Arc<[(H256, TxStatus)]>>) {
        self.full_listeners.push(f);
    }

    /// Add new pending listener to receive notifications.
    pub fn add_pending_listener(&mut self, f: mpsc::UnboundedSender<Arc<[H256]>>) {
        self.pending_listeners.push(f);
    }

    /// Notify listeners about all currently transactions.
    pub fn notify(&mut self) {
        if self.tx_statuses.is_empty() {
            return;
        }

        let to_pending_send: Arc<[H256]> = self
            .tx_statuses
            .clone()
            .into_iter()
            .map(|(hash, _)| hash)
            .collect::<Vec<_>>()
            .into();
        self.pending_listeners
            .retain(|listener| listener.unbounded_send(to_pending_send.clone()).is_ok());

        let to_full_send: Arc<[(H256, TxStatus)]> =
            std::mem::replace(&mut self.tx_statuses, Vec::new()).into();
        self.full_listeners
            .retain(|listener| listener.unbounded_send(to_full_send.clone()).is_ok());
    }
}

impl fmt::Debug for TransactionsPoolNotifier {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("TransactionsPoolNotifier")
            .field("full_listeners", &self.full_listeners.len())
            .field("pending_listeners", &self.pending_listeners.len())
            .finish()
    }
}

impl tx_pool::Listener<Transaction> for TransactionsPoolNotifier {
    fn added(&mut self, tx: &Arc<Transaction>, _old: Option<&Arc<Transaction>>) {
        self.tx_statuses.push((tx.hash, TxStatus::Added));
    }

    fn rejected<H: fmt::Debug + fmt::LowerHex>(
        &mut self,
        tx: &Arc<Transaction>,
        _reason: &tx_pool::Error<H>,
    ) {
        self.tx_statuses.push((tx.hash, TxStatus::Rejected));
    }

    fn dropped(&mut self, tx: &Arc<Transaction>, _new: Option<&Transaction>) {
        self.tx_statuses.push((tx.hash, TxStatus::Dropped));
    }

    fn invalid(&mut self, tx: &Arc<Transaction>) {
        self.tx_statuses.push((tx.hash, TxStatus::Invalid));
    }

    fn canceled(&mut self, tx: &Arc<Transaction>) {
        self.tx_statuses.push((tx.hash, TxStatus::Canceled));
    }

    fn culled(&mut self, tx: &Arc<Transaction>) {
        self.tx_statuses.push((tx.hash, TxStatus::Culled));
    }
}

#[cfg(test)]
mod tests;
