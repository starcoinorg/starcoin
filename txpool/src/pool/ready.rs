// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transaction Readiness indicator
//!
//! Transaction readiness is responsible for indicating if
//! particular transaction can be included in the block.
//!
//! Regular transactions are ready iff the current state nonce
//! (obtained from `NonceClient`) equals to the transaction nonce.
//!
//! Let's define `S = state nonce`. Transactions are processed
//! in order, so we first include transaction with nonce `S`,
//! but then we are able to include the one with `S + 1` nonce.
//! So bear in mind that transactions can be included in chains
//! and their readiness is dependent on previous transactions from
//! the same sender.
//!
//! There are three possible outcomes:
//! - The transaction is old (stalled; state nonce > transaction nonce)
//! - The transaction is ready (current; state nonce == transaction nonce)
//! - The transaction is not ready yet (future; state nonce < transaction nonce)
//!
//! NOTE The transactions are always checked for readines in order they are stored within the queue.
//! First `Readiness::Future` response also causes all subsequent transactions from the same sender
//! to be marked as `Future`.

use std::{cmp, collections::HashMap};

use tx_pool::{self, VerifiedTransaction as PoolVerifiedTransaction};
use types::{account_address::AccountAddress as Address, transaction};

use super::{client::AccountSeqNumberClient, SeqNumber, VerifiedTransaction};

/// Checks readiness of transactions by comparing the nonce to state nonce.
#[derive(Debug)]
pub struct State<C> {
    nonces: HashMap<Address, SeqNumber>,
    state: C,
    max_seq_number: Option<SeqNumber>,
    stale_id: Option<usize>,
}

impl<C> State<C> {
    /// Create new State checker, given client interface.
    pub fn new(state: C, stale_id: Option<usize>) -> Self {
        State {
            nonces: Default::default(),
            state,
            // disable the feature for now.
            max_seq_number: None,
            stale_id,
        }
    }
}

impl<C: AccountSeqNumberClient> tx_pool::Ready<VerifiedTransaction> for State<C> {
    fn is_ready(&mut self, tx: &VerifiedTransaction) -> tx_pool::Readiness {
        // Check max seq number
        match self.max_seq_number {
            Some(nonce) if tx.transaction.sequence_number() > nonce => {
                return tx_pool::Readiness::Future;
            }
            _ => {}
        }

        let sender = tx.sender();
        let state = &self.state;
        if !self.nonces.contains_key(sender) {
            let state_nonce = state.account_seq_number(sender);
            self.nonces.insert(*sender, state_nonce);
        }
        let nonce = self
            .nonces
            .get_mut(sender)
            .expect("sender nonce should exists");
        match tx.transaction.sequence_number().cmp(nonce) {
            // Before marking as future check for stale ids
            cmp::Ordering::Greater => match self.stale_id {
                Some(id) if tx.insertion_id() < id => tx_pool::Readiness::Stale,
                _ => tx_pool::Readiness::Future,
            },
            cmp::Ordering::Less => tx_pool::Readiness::Stale,
            cmp::Ordering::Equal => {
                *nonce = nonce.saturating_add(1);
                tx_pool::Readiness::Ready
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Expiration {
    now: u64,
}
impl Expiration {
    /// Create a new expiration checker given current UTC timestamp.
    pub fn new(now: u64) -> Self {
        Expiration { now }
    }
}

impl tx_pool::Ready<VerifiedTransaction> for Expiration {
    fn is_ready(&mut self, tx: &VerifiedTransaction) -> tx_pool::Readiness {
        if tx.transaction.transaction.expiration_timestamp_secs() <= self.now {
            tx_pool::Readiness::Stale
        } else {
            tx_pool::Readiness::Ready
        }
    }
}

/// Checks readines of Pending transactions by comparing it with current time and block number.
#[derive(Debug)]
pub struct Condition {
    block_number: u64,
    now: u64,
}

impl Condition {
    /// Create a new condition checker given current block number and UTC timestamp.
    pub fn new(block_number: u64, now: u64) -> Self {
        Condition { block_number, now }
    }
}

impl tx_pool::Ready<VerifiedTransaction> for Condition {
    fn is_ready(&mut self, tx: &VerifiedTransaction) -> tx_pool::Readiness {
        match tx.transaction.condition {
            Some(transaction::Condition::Number(block)) if block > self.block_number => {
                tx_pool::Readiness::Future
            }
            Some(transaction::Condition::Timestamp(time)) if time > self.now => {
                tx_pool::Readiness::Future
            }
            _ => tx_pool::Readiness::Ready,
        }
    }
}

/// Readiness checker that only relies on nonce cache (does actually go to state).
///
/// Checks readiness of transactions by comparing the nonce to state nonce. If nonce
/// isn't found in provided state nonce store, defaults to the tx nonce and updates
/// the nonce store. Useful for using with a state nonce cache when false positives are allowed.
pub struct OptionalState<C> {
    nonces: HashMap<Address, SeqNumber>,
    state: C,
}

impl<C> OptionalState<C> {
    pub fn new(state: C) -> Self {
        OptionalState {
            nonces: Default::default(),
            state,
        }
    }
}

impl<C: Fn(&Address) -> Option<SeqNumber>> tx_pool::Ready<VerifiedTransaction>
    for OptionalState<C>
{
    fn is_ready(&mut self, tx: &VerifiedTransaction) -> tx_pool::Readiness {
        let sender = tx.sender();
        let state = &self.state;
        let nonce = self.nonces.entry(*sender).or_insert_with(|| {
            let state_nonce: Option<SeqNumber> = state(sender);
            state_nonce.unwrap_or_else(|| tx.transaction.sequence_number())
        });
        match tx.transaction.sequence_number().cmp(nonce) {
            cmp::Ordering::Greater => tx_pool::Readiness::Future,
            cmp::Ordering::Less => tx_pool::Readiness::Stale,
            cmp::Ordering::Equal => {
                *nonce = nonce.saturating_add(1);
                tx_pool::Readiness::Ready
            }
        }
    }
}
