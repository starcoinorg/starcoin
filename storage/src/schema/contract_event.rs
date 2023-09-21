use crate::CONTRACT_EVENT_PREFIX_NAME;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_types::contract_event::ContractEvent as ContractEventType;

define_schema!(
    ContractEvent,
    HashValue,
    Vec<ContractEventType>,
    CONTRACT_EVENT_PREFIX_NAME
);

impl KeyCodec<ContractEvent> for HashValue {
    fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<ContractEvent> for Vec<ContractEventType> {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}
