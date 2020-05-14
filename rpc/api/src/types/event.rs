use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::EventKey;
use starcoin_types::language_storage::TypeTag;
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub block_hash: Option<HashValue>,
    pub block_number: Option<BlockNumber>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u64>,

    pub data: Vec<u8>,
    pub type_tags: TypeTag,
    #[serde(
        deserialize_with = "deserialize_event_key",
        serialize_with = "serialize_event_key"
    )]
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

pub fn serialize_event_key<S>(key: &EventKey, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(format!("{:#x}", key).as_str())
}
pub fn deserialize_event_key<'de, D>(d: D) -> Result<EventKey, D::Error>
where
    D: Deserializer<'de>,
{
    struct EventKeyVisitor;

    impl<'de> serde::de::Visitor<'de> for EventKeyVisitor {
        type Value = EventKey;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("EventKey in hex string")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let b = hex::decode(v.as_bytes()).map_err(E::custom)?;
            EventKey::try_from(b.as_slice()).map_err(E::custom)
        }
    }
    d.deserialize_str(EventKeyVisitor)
}
