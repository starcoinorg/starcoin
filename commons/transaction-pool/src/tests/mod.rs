// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod helpers;
mod tx_builder;

use self::{
    helpers::{DummyScoring, NonceReady},
    tx_builder::TransactionBuilder,
};
use super::*;
use crypto::hash::HashValue;
use std::sync::Arc;
use tokio;
use types::account_address::AccountAddress as Address;

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    pub hash: HashValue,
    pub nonce: u64,
    pub gas_price: u64,
    pub gas: u64,
    pub sender: Address,
    pub mem_usage: usize,
}

impl VerifiedTransaction for Transaction {
    type Hash = HashValue;
    type Sender = Address;

    fn hash(&self) -> &Self::Hash {
        &self.hash
    }

    fn mem_usage(&self) -> usize {
        self.mem_usage
    }

    fn sender(&self) -> &Self::Sender {
        &self.sender
    }
}

pub type SharedTransaction = Arc<Transaction>;

type TestPool = Pool<Transaction, DummyScoring>;

impl TestPool {
    pub fn with_limit(max_count: usize) -> Self {
        Self::with_options(Options {
            max_count,
            ..Default::default()
        })
    }
}

async fn import<S: Scoring<Transaction>, L: Listener<Transaction>>(
    txq: &mut Pool<Transaction, S, L>,
    tx: Transaction,
) -> Result<Arc<Transaction>, Error<<Transaction as VerifiedTransaction>::Hash>> {
    txq.import(tx, &mut DummyScoring::default()).await
}

#[tokio::test]
async fn should_clear_queue() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    assert_eq!(
        txq.light_status(),
        LightStatus {
            mem_usage: 0,
            transaction_count: 0,
            senders: 0
        }
    );
    let tx1 = b.tx().nonce(0).new();
    let tx2 = b.tx().nonce(1).mem_usage(1).new();

    // add
    import(&mut txq, tx1).await.unwrap();
    import(&mut txq, tx2).await.unwrap();
    assert_eq!(
        txq.light_status(),
        LightStatus {
            mem_usage: 1,
            transaction_count: 2,
            senders: 1
        }
    );

    // when
    txq.clear();

    // then
    assert_eq!(
        txq.light_status(),
        LightStatus {
            mem_usage: 0,
            transaction_count: 0,
            senders: 0
        }
    );
}

#[tokio::test]
async fn should_not_allow_same_transaction_twice() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    let tx1 = b.tx().nonce(0).new();
    let tx2 = b.tx().nonce(0).new();

    // when
    import(&mut txq, tx1).await.unwrap();
    import(&mut txq, tx2).await.unwrap_err();

    // then
    assert_eq!(txq.light_status().transaction_count, 1);
}

#[tokio::test]
async fn should_replace_transaction() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    let tx1 = b.tx().nonce(0).gas_price(1).new();
    let tx2 = b.tx().nonce(0).gas_price(2).new();

    // when
    import(&mut txq, tx1).await.unwrap();
    import(&mut txq, tx2).await.unwrap();

    // then
    assert_eq!(txq.light_status().transaction_count, 1);
}

#[tokio::test]
async fn should_reject_if_above_count() {
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_options(Options {
        max_count: 1,
        ..Default::default()
    });

    // Reject second
    let tx1 = b.tx().nonce(0).new();
    let tx2 = b.tx().nonce(1).new();
    let hash = tx2.hash.clone();
    import(&mut txq, tx1).await.unwrap();
    assert_eq!(
        import(&mut txq, tx2).await.unwrap_err(),
        error::Error::TooCheapToEnter(hash, "0x0".into())
    );
    assert_eq!(txq.light_status().transaction_count, 1);

    txq.clear();

    // Replace first
    let tx1 = b.tx().nonce(0).new();
    let tx2 = b.tx().nonce(0).sender(Address::random()).gas_price(2).new();
    import(&mut txq, tx1).await.unwrap();
    import(&mut txq, tx2).await.unwrap();
    assert_eq!(txq.light_status().transaction_count, 1);
}

#[tokio::test]
async fn should_reject_if_above_mem_usage() {
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_options(Options {
        max_mem_usage: 1,
        ..Default::default()
    });

    // Reject second
    let tx1 = b.tx().nonce(1).mem_usage(1).new();
    let tx2 = b.tx().nonce(2).mem_usage(2).new();
    let hash = tx2.hash.clone();
    import(&mut txq, tx1).await.unwrap();
    assert_eq!(
        import(&mut txq, tx2).await.unwrap_err(),
        error::Error::TooCheapToEnter(hash, "0x0".into())
    );
    assert_eq!(txq.light_status().transaction_count, 1);

    txq.clear();

    // Replace first
    let tx1 = b.tx().nonce(1).mem_usage(1).new();
    let tx2 = b
        .tx()
        .nonce(1)
        .sender(Address::random())
        .gas_price(2)
        .mem_usage(1)
        .new();
    import(&mut txq, tx1).await.unwrap();
    import(&mut txq, tx2).await.unwrap();
    assert_eq!(txq.light_status().transaction_count, 1);
}

#[tokio::test]
async fn should_reject_if_above_sender_count() {
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_options(Options {
        max_per_sender: 1,
        ..Default::default()
    });

    // Reject second
    let tx1 = b.tx().nonce(1).new();
    let tx2 = b.tx().nonce(2).new();
    let hash = tx2.hash.clone();
    import(&mut txq, tx1).await.unwrap();
    assert_eq!(
        import(&mut txq, tx2).await.unwrap_err(),
        error::Error::TooCheapToEnter(hash, "0x0".into())
    );
    assert_eq!(txq.light_status().transaction_count, 1);

    txq.clear();

    // Replace first
    let tx1 = b.tx().nonce(1).new();
    let tx2 = b.tx().nonce(2).gas_price(2).new();
    let hash = tx2.hash.clone();
    import(&mut txq, tx1).await.unwrap();
    // This results in error because we also compare nonces
    assert_eq!(
        import(&mut txq, tx2).await.unwrap_err(),
        error::Error::TooCheapToEnter(hash, "0x0".into())
    );
    assert_eq!(txq.light_status().transaction_count, 1);
}

#[tokio::test]
async fn should_construct_pending() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    let sender1 = Address::random();
    let sender2 = Address::random();
    let tx0 = import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    let tx1 = import(&mut txq, b.tx().nonce(1).gas_price(5).new())
        .await
        .unwrap();

    let tx9 = import(&mut txq, b.tx().sender(sender2).nonce(0).new())
        .await
        .unwrap();

    let tx5 = import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();
    let tx6 = import(&mut txq, b.tx().sender(sender1).nonce(1).new())
        .await
        .unwrap();
    let tx7 = import(&mut txq, b.tx().sender(sender1).nonce(2).new())
        .await
        .unwrap();
    let tx8 = import(&mut txq, b.tx().sender(sender1).nonce(3).gas_price(4).new())
        .await
        .unwrap();

    let tx2 = import(&mut txq, b.tx().nonce(2).new()).await.unwrap();
    // this transaction doesn't get to the block despite high gas price
    // because of block gas limit and simplistic ordering algorithm.
    import(&mut txq, b.tx().nonce(3).gas_price(4).new())
        .await
        .unwrap();
    // gap
    import(&mut txq, b.tx().nonce(5).new()).await.unwrap();

    // gap
    import(&mut txq, b.tx().sender(sender1).nonce(5).new())
        .await
        .unwrap();

    assert_eq!(txq.light_status().transaction_count, 11);
    assert_eq!(
        txq.status(NonceReady::default()).await,
        Status {
            stalled: 0,
            pending: 9,
            future: 2
        }
    );
    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 3,
            pending: 6,
            future: 2
        }
    );

    // when
    let mut pending = txq.pending(NonceReady::default(), 8).await.into_iter();
    assert_eq!(pending.next(), Some(tx0));
    assert_eq!(pending.next(), Some(tx1));
    assert_eq!(pending.next(), Some(tx9));
    assert_eq!(pending.next(), Some(tx5));
    assert_eq!(pending.next(), Some(tx6));
    assert_eq!(pending.next(), Some(tx7));
    assert_eq!(pending.next(), Some(tx8));
    assert_eq!(pending.next(), Some(tx2));
    assert_eq!(pending.next(), None);
}

#[tokio::test]
async fn should_skip_staled_pending_transactions() {
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();

    let _tx0 = import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    let tx2 = import(&mut txq, b.tx().nonce(2).gas_price(5).new())
        .await
        .unwrap();
    let _tx1 = import(&mut txq, b.tx().nonce(1).gas_price(5).new())
        .await
        .unwrap();

    // tx0 and tx1 are Stale, tx2 is Ready
    let mut pending = txq.pending(NonceReady::new(2u64), 10).await.into_iter();

    // tx0 and tx1 should be skipped, tx2 should be the next Ready
    assert_eq!(pending.next(), Some(tx2));
    assert_eq!(pending.next(), None);
}

#[tokio::test]
async fn should_return_unordered_iterator() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();

    let tx0 = import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    let tx1 = import(&mut txq, b.tx().nonce(1).gas_price(5).new())
        .await
        .unwrap();
    let tx2 = import(&mut txq, b.tx().nonce(2).new()).await.unwrap();
    let tx3 = import(&mut txq, b.tx().nonce(3).gas_price(4).new())
        .await
        .unwrap();
    // gap
    import(&mut txq, b.tx().nonce(5).new()).await.unwrap();
    let sender1 = Address::random();
    let tx5 = import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();
    let tx6 = import(&mut txq, b.tx().sender(sender1).nonce(1).new())
        .await
        .unwrap();
    let tx7 = import(&mut txq, b.tx().sender(sender1).nonce(2).new())
        .await
        .unwrap();
    let tx8 = import(&mut txq, b.tx().sender(sender1).nonce(3).gas_price(4).new())
        .await
        .unwrap();
    // gap
    import(&mut txq, b.tx().sender(sender1).nonce(5).new())
        .await
        .unwrap();
    let sender2 = Address::random();
    let tx9 = import(&mut txq, b.tx().sender(sender2).nonce(0).new())
        .await
        .unwrap();
    assert_eq!(txq.light_status().transaction_count, 11);
    assert_eq!(
        txq.status(NonceReady::default()).await,
        Status {
            stalled: 0,
            pending: 9,
            future: 2
        }
    );
    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 3,
            pending: 6,
            future: 2
        }
    );

    // when
    let all: Vec<_> = txq.unordered_pending(NonceReady::default(), 100).await;

    let chain1 = vec![tx0, tx1, tx2, tx3];
    let chain2 = vec![tx5, tx6, tx7, tx8];
    let chain3 = vec![tx9];

    assert_eq!(all.len(), chain1.len() + chain2.len() + chain3.len());

    let mut options = vec![
        vec![chain1.clone(), chain2.clone(), chain3.clone()],
        vec![chain2.clone(), chain1.clone(), chain3.clone()],
        vec![chain2.clone(), chain3.clone(), chain1.clone()],
        vec![chain3.clone(), chain2.clone(), chain1.clone()],
        vec![chain3.clone(), chain1.clone(), chain2.clone()],
        vec![chain1.clone(), chain3.clone(), chain2.clone()],
    ]
    .into_iter()
    .map(|mut v| {
        let mut first = v.pop().unwrap();
        for mut x in v {
            first.append(&mut x);
        }
        first
    });

    assert!(options.any(|opt| all == opt));
}

#[tokio::test]
async fn should_update_scoring_correctly() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    let sender1 = Address::random();
    let sender2 = Address::random();

    let tx9 = import(&mut txq, b.tx().sender(sender2).nonce(0).new())
        .await
        .unwrap();

    let tx5 = import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();
    let tx6 = import(&mut txq, b.tx().sender(sender1).nonce(1).new())
        .await
        .unwrap();
    let tx7 = import(&mut txq, b.tx().sender(sender1).nonce(2).new())
        .await
        .unwrap();
    let tx8 = import(&mut txq, b.tx().sender(sender1).nonce(3).gas_price(4).new())
        .await
        .unwrap();

    let tx0 = import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    let tx1 = import(&mut txq, b.tx().nonce(1).gas_price(5).new())
        .await
        .unwrap();
    let tx2 = import(&mut txq, b.tx().nonce(2).new()).await.unwrap();
    // this transaction doesn't get to the block despite high gas price
    // because of block gas limit and simplistic ordering algorithm.
    import(&mut txq, b.tx().nonce(3).gas_price(4).new())
        .await
        .unwrap();
    // gap
    import(&mut txq, b.tx().nonce(5).new()).await.unwrap();

    // gap
    import(&mut txq, b.tx().sender(sender1).nonce(5).new())
        .await
        .unwrap();

    assert_eq!(txq.light_status().transaction_count, 11);
    assert_eq!(
        txq.status(NonceReady::default()).await,
        Status {
            stalled: 0,
            pending: 9,
            future: 2
        }
    );
    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 3,
            pending: 6,
            future: 2
        }
    );

    txq.update_scores(&Address::default(), ());

    // when
    let mut pending = txq.pending(NonceReady::default(), 8).await.into_iter();

    assert_eq!(pending.next(), Some(tx9));
    assert_eq!(pending.next(), Some(tx5));
    assert_eq!(pending.next(), Some(tx6));
    assert_eq!(pending.next(), Some(tx7));
    assert_eq!(pending.next(), Some(tx8));
    // penalized transactions
    assert_eq!(pending.next(), Some(tx0));
    assert_eq!(pending.next(), Some(tx1));
    assert_eq!(pending.next(), Some(tx2));
    assert_eq!(pending.next(), None);
}

#[tokio::test]
async fn should_remove_transaction() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();

    let tx1 = import(&mut txq, b.tx().nonce(0).new()).await.unwrap();
    let tx2 = import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
    import(&mut txq, b.tx().nonce(2).new()).await.unwrap();
    assert_eq!(txq.light_status().transaction_count, 3);

    // when
    assert!(txq.remove(&tx2.hash(), false).is_some());

    // then
    assert_eq!(txq.light_status().transaction_count, 2);
    let mut pending = txq.pending(NonceReady::default(), 2).await.into_iter();
    assert_eq!(pending.next(), Some(tx1));
    assert_eq!(pending.next(), None);
}

#[tokio::test]
async fn should_cull_stalled_transactions() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();

    import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
    import(&mut txq, b.tx().nonce(3).new()).await.unwrap();
    let sender1 = Address::random();
    import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(1).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(5).new())
        .await
        .unwrap();

    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 2,
            pending: 2,
            future: 2
        }
    );

    // when
    assert_eq!(txq.cull(None, NonceReady::new(1u64)).await, 2);

    // then
    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 0,
            pending: 2,
            future: 2
        }
    );
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 4,
            senders: 2,
            mem_usage: 0
        }
    );
}

#[tokio::test]
async fn should_cull_stalled_transactions_from_a_sender() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();

    import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
    let sender1 = Address::random();
    import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(1).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(2).new())
        .await
        .unwrap();

    assert_eq!(
        txq.status(NonceReady::new(2u64)).await,
        Status {
            stalled: 4,
            pending: 1,
            future: 0
        }
    );

    // when
    let sender = Address::default();
    assert_eq!(txq.cull(Some(&[sender]), NonceReady::new(2u64)).await, 2);

    // then
    assert_eq!(
        txq.status(NonceReady::new(2u64)).await,
        Status {
            stalled: 2,
            pending: 1,
            future: 0
        }
    );
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 3,
            senders: 1,
            mem_usage: 0
        }
    );
}

#[tokio::test]
async fn should_re_insert_after_cull() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    let sender1 = Address::random();
    import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(1).new())
        .await
        .unwrap();
    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 2,
            pending: 2,
            future: 0
        }
    );

    // when
    assert_eq!(txq.cull(None, NonceReady::new(1u64)).await, 2);
    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 0,
            pending: 2,
            future: 0
        }
    );
    import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    import(&mut txq, b.tx().sender(sender1).nonce(0).new())
        .await
        .unwrap();

    assert_eq!(
        txq.status(NonceReady::new(1u64)).await,
        Status {
            stalled: 2,
            pending: 2,
            future: 0
        }
    );
}

#[tokio::test]
async fn should_return_worst_transaction() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::default();
    assert!(txq.worst_transaction().is_none());

    // when
    import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    let sender1 = Address::random();
    import(&mut txq, b.tx().sender(sender1).nonce(0).gas_price(4).new())
        .await
        .unwrap();

    // then
    assert_eq!(txq.worst_transaction().unwrap().gas_price, 4);
}

#[tokio::test]
async fn should_return_is_full() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_limit(2);
    assert!(!txq.is_full());

    // when
    import(&mut txq, b.tx().nonce(0).gas_price(110).new())
        .await
        .unwrap();
    assert!(!txq.is_full());
    let sender1 = Address::random();
    import(
        &mut txq,
        b.tx().sender(sender1).nonce(0).gas_price(100).new(),
    )
    .await
    .unwrap();

    // then
    assert!(txq.is_full());
}

#[tokio::test]
async fn should_import_even_if_limit_is_reached_and_should_replace_returns_insert_new() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_scoring(
        DummyScoring::always_insert(),
        Options {
            max_count: 1,
            ..Default::default()
        },
    );
    txq.import(
        b.tx().nonce(0).gas_price(5).new(),
        &mut DummyScoring::always_insert(),
    )
    .await
    .unwrap();
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 1,
            senders: 1,
            mem_usage: 0
        }
    );

    // when
    txq.import(
        b.tx().nonce(1).gas_price(5).new(),
        &mut DummyScoring::always_insert(),
    )
    .await
    .unwrap();

    // then
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 2,
            senders: 1,
            mem_usage: 0
        }
    );
}

#[tokio::test]
async fn should_not_import_even_if_limit_is_reached_and_should_replace_returns_false() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_scoring(
        DummyScoring::default(),
        Options {
            max_count: 1,
            ..Default::default()
        },
    );
    import(&mut txq, b.tx().nonce(0).gas_price(5).new())
        .await
        .unwrap();
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 1,
            senders: 1,
            mem_usage: 0
        }
    );

    // when
    let txn = b.tx().nonce(1).gas_price(5).new();
    let txn_hash = txn.hash().clone();
    let err = import(&mut txq, txn).await.unwrap_err();
    // then
    assert_eq!(err, error::Error::TooCheapToEnter(txn_hash, "0x5".into()));
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 1,
            senders: 1,
            mem_usage: 0
        }
    );
}

#[tokio::test]
async fn should_import_even_if_sender_limit_is_reached() {
    // given
    let b = TransactionBuilder::default();
    let mut txq = TestPool::with_scoring(
        DummyScoring::always_insert(),
        Options {
            max_count: 1,
            max_per_sender: 1,
            ..Default::default()
        },
    );
    txq.import(
        b.tx().nonce(0).gas_price(5).new(),
        &mut DummyScoring::always_insert(),
    )
    .await
    .unwrap();
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 1,
            senders: 1,
            mem_usage: 0
        }
    );

    // when
    txq.import(
        b.tx().nonce(1).gas_price(5).new(),
        &mut DummyScoring::always_insert(),
    )
    .await
    .unwrap();

    // then
    assert_eq!(
        txq.light_status(),
        LightStatus {
            transaction_count: 2,
            senders: 1,
            mem_usage: 0
        }
    );
}

mod listener {
    use super::*;
    use std::{
        fmt,
        sync::{Arc, RwLock},
    };

    #[derive(Default)]
    struct MyListener(pub Arc<RwLock<Vec<&'static str>>>);

    impl Listener<Transaction> for MyListener {
        fn added(&mut self, _tx: &SharedTransaction, old: Option<&SharedTransaction>) {
            self.0
                .write()
                .unwrap()
                .push(if old.is_some() { "replaced" } else { "added" });
        }

        fn rejected<H: fmt::Debug + fmt::LowerHex>(
            &mut self,
            _tx: &SharedTransaction,
            _reason: &error::Error<H>,
        ) {
            self.0.write().unwrap().push("rejected".into());
        }

        fn dropped(&mut self, _tx: &SharedTransaction, _new: Option<&Transaction>) {
            self.0.write().unwrap().push("dropped".into());
        }

        fn invalid(&mut self, _tx: &SharedTransaction) {
            self.0.write().unwrap().push("invalid".into());
        }

        fn canceled(&mut self, _tx: &SharedTransaction) {
            self.0.write().unwrap().push("canceled".into());
        }

        fn culled(&mut self, _tx: &SharedTransaction) {
            self.0.write().unwrap().push("culled".into());
        }
    }

    #[tokio::test]
    async fn insert_transaction() {
        let b = TransactionBuilder::default();
        let listener = MyListener::default();
        let results = listener.0.clone();
        let mut txq = Pool::new(
            listener,
            DummyScoring::default(),
            Options {
                max_per_sender: 1,
                max_count: 2,
                ..Default::default()
            },
        );
        assert!(results.read().unwrap().is_empty());

        // Regular import
        let txn = b.tx().nonce(1).new();
        import(&mut txq, txn.clone()).await.unwrap();
        assert_eq!(*results.read().unwrap(), &["added"]);
        // Already present (no notification)
        import(&mut txq, txn).await.unwrap_err();
        assert_eq!(*results.read().unwrap(), &["added"]);
        // Push out the first one
        import(&mut txq, b.tx().nonce(1).gas_price(1).new())
            .await
            .unwrap();
        assert_eq!(*results.read().unwrap(), &["added", "replaced"]);
        // Reject
        import(&mut txq, b.tx().nonce(1).new()).await.unwrap_err();
        assert_eq!(*results.read().unwrap(), &["added", "replaced", "rejected"]);
        results.write().unwrap().clear();
        // Different sender (accept)
        let sender1 = Address::random();
        let sender2 = Address::random();
        import(&mut txq, b.tx().sender(sender1).nonce(1).gas_price(2).new())
            .await
            .unwrap();
        assert_eq!(*results.read().unwrap(), &["added"]);
        // Third sender push out low gas price
        import(&mut txq, b.tx().sender(sender2).nonce(1).gas_price(4).new())
            .await
            .unwrap();
        assert_eq!(*results.read().unwrap(), &["added", "dropped", "added"]);
        // Reject (too cheap)
        import(&mut txq, b.tx().sender(sender2).nonce(1).gas_price(2).new())
            .await
            .unwrap_err();
        assert_eq!(
            *results.read().unwrap(),
            &["added", "dropped", "added", "rejected"]
        );

        assert_eq!(txq.light_status().transaction_count, 2);
    }

    #[tokio::test]
    async fn remove_transaction() {
        let b = TransactionBuilder::default();
        let listener = MyListener::default();
        let results = listener.0.clone();
        let mut txq = Pool::new(listener, DummyScoring::default(), Options::default());

        // insert
        let tx1 = import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
        let tx2 = import(&mut txq, b.tx().nonce(2).new()).await.unwrap();

        // then
        txq.remove(&tx1.hash(), false);
        assert_eq!(*results.read().unwrap(), &["added", "added", "canceled"]);
        txq.remove(&tx2.hash(), true);
        assert_eq!(
            *results.read().unwrap(),
            &["added", "added", "canceled", "invalid"]
        );
        assert_eq!(txq.light_status().transaction_count, 0);
    }

    #[tokio::test]
    async fn clear_queue() {
        let b = TransactionBuilder::default();
        let listener = MyListener::default();
        let results = listener.0.clone();
        let mut txq = Pool::new(listener, DummyScoring::default(), Options::default());

        // insert
        import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
        import(&mut txq, b.tx().nonce(2).new()).await.unwrap();

        // when
        txq.clear();

        // then
        assert_eq!(
            *results.read().unwrap(),
            &["added", "added", "dropped", "dropped"]
        );
    }

    #[tokio::test]
    async fn cull_stalled() {
        let b = TransactionBuilder::default();
        let listener = MyListener::default();
        let results = listener.0.clone();
        let mut txq = Pool::new(listener, DummyScoring::default(), Options::default());

        // insert
        import(&mut txq, b.tx().nonce(1).new()).await.unwrap();
        import(&mut txq, b.tx().nonce(2).new()).await.unwrap();

        // when
        txq.cull(None, NonceReady::new(3u64)).await;

        // then
        assert_eq!(
            *results.read().unwrap(),
            &["added", "added", "culled", "culled"]
        );
    }
}
