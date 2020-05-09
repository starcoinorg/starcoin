use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::event::EventKey;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub data: Vec<u8>,
    pub block_hash: Option<HashValue>,
    pub transaction_hash: Option<HashValue>,
    pub event_key: EventKey,
    pub event_seq_number: u64,
}
