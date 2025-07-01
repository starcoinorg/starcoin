//! Blockchain filter

use crate::account_address::AccountAddress;
use crate::block::BlockNumber;
use crate::contract_event::StcContractEvent;
use crate::event::StcEventKey;
use crate::language_storage::StcTypeTag;
use starcoin_vm2_vm_types::language_storage::type_tag_match as type_tag_match_v2;
use starcoin_vm_types::language_storage::type_tag_match;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterType {
    VM1,
    VM2,
    Both,
}

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
    pub event_keys: Vec<StcEventKey>,

    /// Account addresses which event comes from.
    /// match if event belongs to any og the addresses.
    /// if `addrs` is empty, event always match.
    pub addrs: Vec<AccountAddress>,
    /// type tags of the event.
    /// match if the event is any type of the type tags.
    /// if `type_tags` is empty, event always match.
    pub type_tags: Vec<StcTypeTag>,

    /// Events limit
    ///
    /// If None, return all events
    /// If specified, should only return *last* `n` events.
    pub limit: Option<usize>,
    /// return events in reverse order.
    pub reverse: bool,
    /// return events of certain type.
    pub filter_type: FilterType,
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
            filter_type: FilterType::VM2,
        }
    }
}

impl Filter {
    pub fn matching(&self, block_number: BlockNumber, e: &StcContractEvent) -> bool {
        // quick path for vm2_only or vm1_only filter
        match self.filter_type {
            FilterType::VM1 if e.is_v2() => return false,
            FilterType::VM2 if e.is_v1() => return false,
            _ => {}
        }

        let creator_address = match e {
            StcContractEvent::V1(event) => event.key().get_creator_address(),
            StcContractEvent::V2(event) => {
                AccountAddress::new(event.event_key().get_creator_address().into_bytes())
            }
        };
        if self.from_block <= block_number
            && block_number <= self.to_block
            && (self.event_keys.is_empty() || self.event_keys.contains(&e.key()))
            && (self.addrs.is_empty() || self.addrs.contains(&creator_address))
        {
            if self.type_tags.is_empty() {
                return true;
            } else {
                for filter_type_tag in &self.type_tags {
                    match (filter_type_tag, e.type_tag()) {
                        (StcTypeTag::V1(filter_tag), StcTypeTag::V1(event_tag)) => {
                            if type_tag_match(filter_tag, &event_tag) {
                                return true;
                            }
                        }
                        (StcTypeTag::V2(filter_tag), StcTypeTag::V2(event_tag)) => {
                            if type_tag_match_v2(filter_tag, &event_tag) {
                                return true;
                            }
                        }
                        (StcTypeTag::V2(_), StcTypeTag::V1(_)) => continue,
                        (StcTypeTag::V1(_), StcTypeTag::V2(_)) => continue,
                    }
                }
            }
        }
        false
    }
}
