use crate::BLOCK_HEADER_PREFIX_NAME;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_types::block::BlockHeader as BlockHeaderType;

define_schema!(
    BlockHeader,
    HashValue,
    BlockHeaderType,
    BLOCK_HEADER_PREFIX_NAME
);

impl KeyCodec<BlockHeader> for HashValue {
    fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<BlockHeader> for BlockHeaderType {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}
