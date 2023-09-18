use crate::block::FailedBlock as FailedBlockType;
use crate::FAILED_BLOCK_PREFIX_NAME;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};

define_schema!(
    FailedBlock,
    HashValue,
    FailedBlockType,
    FAILED_BLOCK_PREFIX_NAME
);

impl KeyCodec<FailedBlock> for HashValue {
    fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<FailedBlock> for FailedBlockType {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}
