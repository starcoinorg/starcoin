//! Blockchain filter

use crate::block::BlockNumber;
use crate::contract_event::ContractEvent;
use crate::event::EventKey;

#[derive(Debug, PartialEq)]
pub struct Filter {
    /// Blockchain will be searched from this block.
    pub from_block: BlockNumber,
    /// Till this block.
    pub to_block: BlockNumber,

    /// Search events.
    ///
    /// If empty, match all.
    /// If specified, event must produced from one of the event keys.
    pub event_keys: Vec<EventKey>,
    /// Events limit
    ///
    /// If None, return all events
    /// If specified, should only return *last* `n` events.
    pub limit: Option<usize>,
}

impl Filter {
    pub fn matching(&self, _e: &ContractEvent) -> bool {
        true
        // TODO: wait contract event done
        // match (&self.block_hash, &e.block_hash) {
        //     (None, _) => true,
        //     (Some(h),) => false,
        //     _ => true,
        // }
    }
}
