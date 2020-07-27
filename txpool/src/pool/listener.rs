// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, sync::Arc};

use super::{TxStatus, VerifiedTransaction as Transaction};
use common_crypto::hash::HashValue as H256;
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
            "Txn added. [{hash:?}] Sender: {sender}, nonce: {nonce}, gasPrice: {gas_price}, gas: {gas}, dataLen: {data:?}))",
            hash = tx.hash(),
            sender = tx.sender(),
            nonce = tx.signed().sequence_number(),
            gas_price = tx.signed().gas_unit_price(),
            gas = tx.signed().max_gas_amount(),
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
    full_listeners: Vec<mpsc::UnboundedSender<Arc<Vec<(H256, TxStatus)>>>>,
    pending_listeners: Vec<mpsc::UnboundedSender<Arc<Vec<H256>>>>,
    tx_statuses: Vec<(H256, TxStatus)>,
}

impl TransactionsPoolNotifier {
    /// Add new full listener to receive notifications.
    pub fn add_full_listener(&mut self, f: mpsc::UnboundedSender<Arc<Vec<(H256, TxStatus)>>>) {
        self.full_listeners.push(f);
    }

    /// Add new pending listener to receive notifications.
    pub fn add_pending_listener(&mut self, f: mpsc::UnboundedSender<Arc<Vec<H256>>>) {
        self.pending_listeners.push(f);
    }

    /// Notify listeners about all currently transactions.
    pub fn notify(&mut self) {
        if self.tx_statuses.is_empty() {
            return;
        }

        let to_pending_send: Arc<Vec<H256>> = Arc::new(
            self.tx_statuses
                .clone()
                .into_iter()
                .map(|(hash, _)| hash)
                .collect(),
        );
        self.pending_listeners
            .retain(|listener| listener.unbounded_send(to_pending_send.clone()).is_ok());

        let to_full_send = Arc::new(std::mem::replace(&mut self.tx_statuses, Vec::new()));
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
mod tests {
    use super::*;
    use common_crypto::{ed25519, Uniform};
    use rand::SeedableRng;

    use tx_pool::Listener;
    use types::{
        account_address::AccountAddress,
        transaction,
        transaction::helpers::get_current_timestamp,
        transaction::{Script, TransactionPayload},
    };

    #[test]
    fn should_notify_listeners() {
        // given
        let (full_sender, mut full_receiver) = mpsc::unbounded();
        let (pending_sender, mut pending_receiver) = mpsc::unbounded();

        let mut tx_listener = TransactionsPoolNotifier::default();
        tx_listener.add_full_listener(full_sender);
        tx_listener.add_pending_listener(pending_sender);

        // when
        let tx = new_tx();
        tx_listener.added(&tx, None);

        // then
        tx_listener.notify();
        let full_res = full_receiver.try_next().unwrap();
        let pending_res = pending_receiver.try_next().unwrap();
        assert_eq!(
            full_res,
            Some(Arc::new(vec![(*tx.hash(), TxStatus::Added)]))
        );
        assert_eq!(pending_res, Some(Arc::new(vec![*tx.hash()])));
    }

    fn new_tx() -> Arc<Transaction> {
        let raw = transaction::RawUserTransaction::new(
            AccountAddress::random(),
            4,
            TransactionPayload::Script(Script::new(vec![1, 2, 3], vec![], vec![])),
            100_000,
            10,
            get_current_timestamp() + 60,
            ChainId::test(),
        );
        let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
        let private_key = ed25519::Ed25519PrivateKey::generate(&mut rng);
        let public_key = (&private_key).into();

        let signed = raw.sign(&private_key, public_key).unwrap().into_inner();
        Arc::new(Transaction::from_pending_block_transaction(signed))
    }
}
