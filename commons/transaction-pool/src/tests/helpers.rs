// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{cmp, collections::HashMap};

use super::Transaction;
use crate::{pool, scoring, Readiness, Ready, ReplaceTransaction, Scoring, ShouldReplace};
use types::account_address::AccountAddress as Sender;

#[derive(Debug, Default)]
pub struct DummyScoring {
    always_insert: bool,
}

impl DummyScoring {
    pub fn always_insert() -> Self {
        DummyScoring {
            always_insert: true,
        }
    }
}

impl Scoring<Transaction> for DummyScoring {
    type Event = ();
    type Score = u64;

    fn compare(&self, old: &Transaction, new: &Transaction) -> cmp::Ordering {
        old.nonce.cmp(&new.nonce)
    }

    fn choose(&self, old: &Transaction, new: &Transaction) -> scoring::Choice {
        if old.nonce == new.nonce {
            if new.gas_price > old.gas_price {
                scoring::Choice::ReplaceOld
            } else {
                scoring::Choice::RejectNew
            }
        } else {
            scoring::Choice::InsertNew
        }
    }

    fn update_scores(
        &self,
        txs: &[pool::Transaction<Transaction>],
        scores: &mut [Self::Score],
        change: scoring::Change,
    ) {
        if let scoring::Change::Event(_) = change {
            // In case of event reset all scores to 0
            for i in 0..txs.len() {
                scores[i] = 0;
            }
        } else {
            // Set to a gas price otherwise
            for i in 0..txs.len() {
                scores[i] = txs[i].gas_price;
            }
        }
    }

    fn should_ignore_sender_limit(&self, _new: &Transaction) -> bool {
        self.always_insert
    }
}

#[async_trait]
impl ShouldReplace<Transaction> for DummyScoring {
    async fn should_replace<'a>(
        &self,
        old: ReplaceTransaction<'a, Transaction>,
        new: ReplaceTransaction<'a, Transaction>,
    ) -> scoring::Choice {
        if self.always_insert {
            scoring::Choice::InsertNew
        } else if new.gas_price > old.gas_price {
            scoring::Choice::ReplaceOld
        } else {
            scoring::Choice::RejectNew
        }
    }
}

#[derive(Default)]
pub struct NonceReady(HashMap<Sender, u64>, u64);

impl NonceReady {
    pub fn new<T: Into<u64>>(min: T) -> Self {
        let mut n = NonceReady::default();
        n.1 = min.into();
        n
    }
}
#[async_trait(?Send)]
impl Ready<Transaction> for NonceReady {
    async fn is_ready(&mut self, tx: &Transaction) -> Readiness {
        let min = self.1;
        let nonce = self.0.entry(tx.sender).or_insert_with(|| min);
        match tx.nonce.cmp(nonce) {
            cmp::Ordering::Greater => Readiness::Future,
            cmp::Ordering::Equal => {
                *nonce += 1;
                Readiness::Ready
            }
            cmp::Ordering::Less => Readiness::Stale,
        }
    }
}
