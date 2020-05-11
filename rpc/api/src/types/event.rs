use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::EventKey;
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub data: Vec<u8>,
    pub block_hash: Option<HashValue>,
    pub block_number: Option<BlockNumber>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u64>,
    pub event_key: EventKey,
    pub event_seq_number: u64,
}

impl Event {
    pub fn new(
        block_hash: Option<HashValue>,
        block_number: Option<BlockNumber>,
        transaction_hash: Option<HashValue>,
        transaction_index: Option<u64>,
        _contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            // TODO: fill me
            data: vec![],
            event_key: EventKey::random(),
            event_seq_number: 0,
        }
    }
}
