use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::EventKey;
use starcoin_types::language_storage::TypeTag;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub block_hash: Option<HashValue>,
    pub block_number: Option<BlockNumber>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u64>,

    pub data: Vec<u8>,
    pub type_tags: TypeTag,
    pub event_key: EventKey,
    pub event_seq_number: u64,
}

impl Event {
    pub fn new(
        block_hash: Option<HashValue>,
        block_number: Option<BlockNumber>,
        transaction_hash: Option<HashValue>,
        transaction_index: Option<u64>,
        contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            data: contract_event.event_data().to_vec(),
            type_tags: contract_event.type_tag().clone(),
            event_key: contract_event.key().clone(),
            event_seq_number: contract_event.sequence_number(),
        }
    }
}
