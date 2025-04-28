use starcoin_types::block::BlockNumber;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_vm_types::contract_event::ContractEvent;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ContractEventNotification(pub (HashValue, Arc<[Event]>));

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub block_hash: HashValue,
    pub block_number: BlockNumber,
    pub transaction_hash: HashValue,
    // txn index in block
    pub transaction_index: Option<u32>,
    pub transaction_global_index: Option<u64>,
    pub event_index: Option<u32>,
    pub contract_event: ContractEvent,
}

impl Event {
    pub fn new(
        block_hash: HashValue,
        block_number: BlockNumber,
        transaction_hash: HashValue,
        transaction_index: Option<u32>,
        transaction_global_index: Option<u64>,
        event_index: Option<u32>,
        contract_event: ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            transaction_global_index,
            event_index,
            contract_event,
        }
    }
}
