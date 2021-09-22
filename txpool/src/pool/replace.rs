// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Replacing Transactions
//!
//! When queue limits are reached, a new transaction may replace one already
//! in the pool. The decision whether to reject, replace or retain both is
//! delegated to an implementation of `ShouldReplace`.
//!
//! Here we decide based on the sender, the nonce and gas price, and finally
//! on the `Readiness` of the transactions when comparing them

use crate::pool::{client, ScoredTransaction};
use std::cmp;
use tx_pool::{
    self,
    scoring::{Choice, Scoring},
    ReplaceTransaction, VerifiedTransaction,
};
use types::account_address::AccountAddress as Address;

/// Choose whether to replace based on the sender, the score and finally the
/// `Readiness` of the transactions being compared.
#[derive(Debug)]
pub struct ReplaceByScoreAndReadiness<S, C> {
    scoring: S,
    client: C,
}

impl<S, C> ReplaceByScoreAndReadiness<S, C> {
    /// Create a new `ReplaceByScoreAndReadiness`
    pub fn new(scoring: S, client: C) -> Self {
        ReplaceByScoreAndReadiness { scoring, client }
    }
}

impl<T, S, C> tx_pool::ShouldReplace<T> for ReplaceByScoreAndReadiness<S, C>
where
    T: VerifiedTransaction<Sender = Address> + ScoredTransaction + PartialEq,
    S: Scoring<T> + Sync,
    C: client::AccountSeqNumberClient,
{
    fn should_replace<'r>(
        &self,
        old: &'r ReplaceTransaction<'r, T>,
        new: &'r ReplaceTransaction<'r, T>,
    ) -> Choice {
        let both_local = old.priority().is_local() && new.priority().is_local();
        if old.sender() == new.sender() {
            // prefer earliest transaction
            match new.seq_number().cmp(&old.seq_number()) {
                cmp::Ordering::Equal => self.scoring.choose(old, new),
                _ if both_local => Choice::InsertNew,
                cmp::Ordering::Less => Choice::ReplaceOld,
                cmp::Ordering::Greater => Choice::RejectNew,
            }
        } else if both_local {
            Choice::InsertNew
        } else {
            let old_score = (old.priority(), old.gas_price());
            let new_score = (new.priority(), new.gas_price());
            if new_score <= old_score {
                Choice::RejectNew
            } else {
                // Check if this is a replacement transaction.
                //
                // With replacement transactions we can safely return `InsertNew` here, because
                // we don't need to remove `old` (worst transaction in the pool) since `new` will replace
                // some other transaction in the pool so we will never go above limit anyway.
                if let Some(txs) = new.pooled_by_sender {
                    if let Ok(index) = txs.binary_search_by(|old| self.scoring.compare(old, new)) {
                        return match self.scoring.choose(&txs[index], new) {
                            Choice::ReplaceOld => Choice::InsertNew,
                            choice => choice,
                        };
                    }
                }

                Choice::ReplaceOld
                // let state = &self.client;
                // // calculate readiness based on state nonce + pooled txs from same sender
                // let is_ready = |replace: &ReplaceTransaction<T>| {
                //     let mut nonce = state.account_nonce(replace.sender());
                //     if let Some(txs) = replace.pooled_by_sender {
                //         for tx in txs.iter() {
                //             if nonce == tx.nonce()
                //                 && (&tx.transaction != &replace.transaction.transaction)
                //             {
                //                 nonce = nonce.saturating_add(1)
                //             } else {
                //                 break;
                //             }
                //         }
                //     }
                //     nonce == replace.nonce()
                // };
                //
                // if !is_ready(new) && is_ready(old) {
                //     // prevent a ready transaction being replace by a non-ready transaction
                //     Choice::RejectNew
                // } else {
                //     Choice::ReplaceOld
                // }
            }
        }
    }
}
