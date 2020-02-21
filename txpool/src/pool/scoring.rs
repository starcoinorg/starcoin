use std::cmp;

use super::{verifier, GasPrice, PrioritizationStrategy, ScoredTransaction, VerifiedTransaction};
use tx_pool::{self, scoring};
/// Transaction with the same (sender, nonce) can be replaced only if
/// `new_gas_price >= old_gas_price + old_gas_price >> SHIFT`
const GAS_PRICE_BUMP_SHIFT: usize = 3; // 2 = 25%, 3 = 12.5%, 4 = 6.25%

/// Calculate minimal gas price requirement.
#[inline]
fn bump_gas_price(old_gp: GasPrice) -> GasPrice {
    old_gp.saturating_add(old_gp >> GAS_PRICE_BUMP_SHIFT as u64)
}

/// Simple, gas-price based scoring for transactions.
///
/// NOTE: Currently penalization does not apply to new transactions that enter the pool.
/// We might want to store penalization status in some persistent state.
#[derive(Debug, Clone)]
pub struct NonceAndGasPrice(pub PrioritizationStrategy);

impl NonceAndGasPrice {
    /// Decide if the transaction should even be considered into the pool (if the pool is full).
    ///
    /// Used by Verifier to quickly reject transactions that don't have any chance to get into the pool later on,
    /// and save time on more expensive checks like sender recovery, etc.
    ///
    /// NOTE The method is never called for zero-gas-price transactions or local transactions
    /// (such transactions are always considered to the pool and potentially rejected later on)
    pub fn should_reject_early(&self, old: &VerifiedTransaction) -> bool {
        todo!()
    }

    //    pub fn should_reject_early(
    //        &self,
    //        old: &VerifiedTransaction,
    //        new: &verifier::Transaction,
    //    ) -> bool {
    //        if old.priority().is_local() {
    //            return true;
    //        }
    //
    //        &old.transaction.gas_price > new.gas_price()
    //    }
}

impl<P> tx_pool::Scoring<P> for NonceAndGasPrice
where
    P: ScoredTransaction + tx_pool::VerifiedTransaction,
{
    type Score = u64;
    type Event = ();

    fn compare(&self, old: &P, other: &P) -> cmp::Ordering {
        old.nonce().cmp(&other.nonce())
    }

    fn choose(&self, old: &P, new: &P) -> scoring::Choice {
        if old.nonce() != new.nonce() {
            return scoring::Choice::InsertNew;
        }

        let old_gp = old.gas_price();
        let new_gp = new.gas_price();

        let min_required_gp = bump_gas_price(old_gp);

        match min_required_gp.cmp(&new_gp) {
            cmp::Ordering::Greater => scoring::Choice::RejectNew,
            _ => scoring::Choice::ReplaceOld,
        }
    }

    fn update_scores(
        &self,
        txs: &[tx_pool::Transaction<P>],
        scores: &mut [Self::Score],
        change: scoring::Change,
    ) {
        use self::scoring::Change;

        match change {
            Change::Culled(_) => {}
            Change::RemovedAt(_) => {}
            Change::InsertedAt(i) | Change::ReplacedAt(i) => {
                assert!(i < txs.len());
                assert!(i < scores.len());

                scores[i] = txs[i].transaction.gas_price();
                let boost = match txs[i].priority() {
                    super::Priority::Local => 15,
                    super::Priority::Retracted => 10,
                    super::Priority::Regular => 0,
                };
                scores[i] = scores[i] << boost;
            }
            // We are only sending an event in case of penalization.
            // So just lower the priority of all non-local transactions.
            Change::Event(_) => {
                for (score, tx) in scores.iter_mut().zip(txs) {
                    // Never penalize local transactions.
                    if !tx.priority().is_local() {
                        *score = *score >> 3;
                    }
                }
            }
        }
    }

    fn should_ignore_sender_limit(&self, new: &P) -> bool {
        new.priority().is_local()
    }
}
