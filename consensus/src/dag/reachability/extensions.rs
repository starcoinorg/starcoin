use crate::consensusdb::{prelude::StoreResult, schemadb::ReachabilityStoreReader};
use crate::dag::types::interval::Interval;
use starcoin_crypto::hash::HashValue as Hash;

pub(super) trait ReachabilityStoreIntervalExtensions {
    fn interval_children_capacity(&self, block: Hash) -> StoreResult<Interval>;
    fn interval_remaining_before(&self, block: Hash) -> StoreResult<Interval>;
    fn interval_remaining_after(&self, block: Hash) -> StoreResult<Interval>;
}

impl<T: ReachabilityStoreReader + ?Sized> ReachabilityStoreIntervalExtensions for T {
    /// Returns the reachability allocation capacity for children of `block`
    fn interval_children_capacity(&self, block: Hash) -> StoreResult<Interval> {
        // The interval of a block should *strictly* contain the intervals of its
        // tree children, hence we subtract 1 from the end of the range.
        Ok(self.get_interval(block)?.decrease_end(1))
    }

    /// Returns the available interval to allocate for tree children, taken from the
    /// beginning of children allocation capacity
    fn interval_remaining_before(&self, block: Hash) -> StoreResult<Interval> {
        let alloc_capacity = self.interval_children_capacity(block)?;
        match self.get_children(block)?.first() {
            Some(first_child) => {
                let first_alloc = self.get_interval(*first_child)?;
                Ok(Interval::new(
                    alloc_capacity.start,
                    first_alloc.start.checked_sub(1).unwrap(),
                ))
            }
            None => Ok(alloc_capacity),
        }
    }

    /// Returns the available interval to allocate for tree children, taken from the
    /// end of children allocation capacity
    fn interval_remaining_after(&self, block: Hash) -> StoreResult<Interval> {
        let alloc_capacity = self.interval_children_capacity(block)?;
        match self.get_children(block)?.last() {
            Some(last_child) => {
                let last_alloc = self.get_interval(*last_child)?;
                Ok(Interval::new(
                    last_alloc.end.checked_add(1).unwrap(),
                    alloc_capacity.end,
                ))
            }
            None => Ok(alloc_capacity),
        }
    }
}
