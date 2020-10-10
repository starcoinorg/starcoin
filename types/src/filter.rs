//! Blockchain filter

use crate::block::BlockNumber;
use crate::contract_event::ContractEvent;
use crate::event::EventKey;

#[derive(Clone, Debug, PartialEq)]
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
    /// return events in reverse order.
    pub reverse: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            from_block: 0,
            to_block: 0,
            event_keys: vec![],
            limit: None,
            reverse: true,
        }
    }
}

impl Filter {
    pub fn matching(&self, block_number: BlockNumber, e: &ContractEvent) -> bool {
        if self.from_block <= block_number
            && block_number <= self.to_block
            && (self.event_keys.is_empty() || self.event_keys.contains(e.key()))
        {
            return true;
        }
        false
    }
}
