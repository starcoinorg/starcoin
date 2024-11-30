//! Blockchain filter

use starcoin_vm_types::language_storage::type_tag_match;

use crate::account_address::AccountAddress;
use crate::block::BlockNumber;
use crate::contract_event::ContractEvent;
use crate::event::EventKey;
use crate::language_storage::TypeTag;

#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Account addresses which event comes from.
    /// match if event belongs to any og the addresses.
    /// if `addrs` is empty, event always match.
    pub addrs: Vec<AccountAddress>,
    /// type tags of the event.
    /// match if the event is any type of the type tags.
    /// if `type_tags` is empty, event always match.
    pub type_tags: Vec<TypeTag>,

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
            type_tags: vec![],
            addrs: vec![],
            limit: None,
            reverse: true,
        }
    }
}

impl Filter {
    pub fn matching(&self, block_number: BlockNumber, e: &ContractEvent) -> bool {
        let event_key = e.v1().unwrap().key();
        if self.from_block <= block_number
            && block_number <= self.to_block
            && (self.event_keys.is_empty()
                || self.event_keys.contains(event_key)
                    && (self.addrs.is_empty()
                        || self.addrs.contains(&event_key.get_creator_address())))
        {
            if self.type_tags.is_empty() {
                return true;
            } else {
                for filter_type_tag in &self.type_tags {
                    if type_tag_match(filter_type_tag, e.type_tag()) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
