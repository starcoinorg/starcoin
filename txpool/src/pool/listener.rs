// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, sync::Arc};

use super::{TxStatus, VerifiedTransaction as Transaction};
use crypto::hash::HashValue as H256;
use futures_channel::mpsc;
use starcoin_logger::prelude::*;
use transaction_pool as tx_pool;
use tx_pool::VerifiedTransaction;
/// Transaction pool logger.
#[derive(Default, Debug)]
pub struct Logger;

impl tx_pool::Listener<Transaction> for Logger {
    fn added(&mut self, tx: &Arc<Transaction>, old: Option<&Arc<Transaction>>) {
        sl_info!(
            "{action} {hash} {sender} {nonce} {gas_price} {gas} {expiration_timestamp_secs}",
            nonce = tx.signed().sequence_number(),
            gas_price = tx.signed().gas_unit_price(),
            gas = tx.signed().max_gas_amount(),
            expiration_timestamp_secs = tx.signed().expiration_timestamp_secs(),
            sender = tx.sender().to_hex(),
            hash = tx.hash().to_hex(),
            action = "txpool_add",
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
        sl_info!(
            "{action} {hash} {reason}",
            reason = format!("{}", reason),
            hash = tx.hash().to_hex(),
            action = "txpool_reject",
        );
    }

    fn dropped(&mut self, tx: &Arc<Transaction>, new: Option<&Transaction>) {
        match new {
            Some(new) => {
                sl_info!(
                    "{action} {hash} {new_hash}",
                    new_hash = new.hash().to_hex(),
                    hash = tx.hash().to_hex(),
                    action = "txpool_drop",
                )
            }
            None => sl_info!(
                "{action} {hash}",
                hash = tx.hash().to_hex(),
                action = "txpool_drop",
            ),
        }
    }

    fn invalid(&mut self, tx: &Arc<Transaction>) {
        debug!(target: "txqueue", "[{:?}] Marked as invalid by executor.", tx.hash());
    }

    fn canceled(&mut self, tx: &Arc<Transaction>) {
        sl_info!(
            "{action} {hash}",
            hash = tx.hash().to_hex(),
            action = "txpool_cancel",
        );
    }

    fn culled(&mut self, tx: &Arc<Transaction>) {
        sl_info!(
            "{action} {hash}",
            hash = tx.hash().to_hex(),
            action = "txpool_cull",
        );
    }
}

/// Transaction status logger.
#[derive(Default, Debug)]
pub struct StatusLogger;
impl StatusLogger {
    fn log_status(tx: &Arc<Transaction>, status: TxStatus) {
        debug!(
            target: "tx-status",
            "[tx-status] hash: {hash}, status: {status}",
            hash = tx.hash(),
            status = status
        );
    }
}
impl tx_pool::Listener<Transaction> for StatusLogger {
    fn added(&mut self, tx: &Arc<Transaction>, old: Option<&Arc<Transaction>>) {
        Self::log_status(tx, TxStatus::Added);
        if let Some(old) = old {
            Self::log_status(old, TxStatus::Dropped);
        }
    }

    fn rejected<H: fmt::Debug + fmt::LowerHex>(
        &mut self,
        tx: &Arc<Transaction>,
        _reason: &tx_pool::Error<H>,
    ) {
        Self::log_status(tx, TxStatus::Rejected);
    }

    fn dropped(&mut self, tx: &Arc<Transaction>, _new: Option<&Transaction>) {
        Self::log_status(tx, TxStatus::Dropped);
    }

    fn invalid(&mut self, tx: &Arc<Transaction>) {
        Self::log_status(tx, TxStatus::Invalid);
    }

    fn canceled(&mut self, tx: &Arc<Transaction>) {
        Self::log_status(tx, TxStatus::Canceled);
    }

    fn culled(&mut self, tx: &Arc<Transaction>) {
        Self::log_status(tx, TxStatus::Culled);
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

        let to_full_send: Arc<[(H256, TxStatus)]> = std::mem::take(&mut self.tx_statuses).into();
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
