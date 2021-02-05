use super::*;
use crypto::{ed25519, Uniform};
use rand::SeedableRng;

use tx_pool::Listener;
use types::genesis_config::ChainId;
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
    assert_eq!(full_res, Some(vec![(*tx.hash(), TxStatus::Added)].into()));
    assert_eq!(pending_res, Some(vec![*tx.hash()].into()));
}

#[test]
fn test_notify() {
    // given
    let (full_sender, mut full_receiver) = mpsc::unbounded();
    let mut tx_listener = TransactionsPoolNotifier::default();
    tx_listener.add_full_listener(full_sender);

    // rejected
    let tx = new_tx();
    tx_listener.rejected(&tx, &tx_pool::Error::AlreadyImported(tx.hash));
    tx_listener.notify();
    let full_res = full_receiver.try_next().unwrap();
    assert_eq!(
        full_res,
        Some(vec![(*tx.hash(), TxStatus::Rejected)].into())
    );

    // dropped
    tx_listener.dropped(&tx, None);
    tx_listener.notify();
    let full_res = full_receiver.try_next().unwrap();
    assert_eq!(full_res, Some(vec![(*tx.hash(), TxStatus::Dropped)].into()));

    // canceled
    tx_listener.canceled(&tx);
    tx_listener.notify();
    let full_res = full_receiver.try_next().unwrap();
    assert_eq!(
        full_res,
        Some(vec![(*tx.hash(), TxStatus::Canceled)].into())
    );

    // culled
    tx_listener.culled(&tx);
    tx_listener.notify();
    let full_res = full_receiver.try_next().unwrap();
    assert_eq!(full_res, Some(vec![(*tx.hash(), TxStatus::Culled)].into()));

    // invalid
    tx_listener.invalid(&tx);
    tx_listener.notify();
    let full_res = full_receiver.try_next().unwrap();
    assert_eq!(full_res, Some(vec![(*tx.hash(), TxStatus::Invalid)].into()));
}

fn new_tx() -> Arc<Transaction> {
    let raw = transaction::RawUserTransaction::new_with_default_gas_token(
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
