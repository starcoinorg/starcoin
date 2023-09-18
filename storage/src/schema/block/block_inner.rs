use crate::BLOCK_PREFIX_NAME;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::define_schema;
use starcoin_schemadb::schema::{KeyCodec, ValueCodec};
use starcoin_types::block::Block;

define_schema!(BlockInner, HashValue, Block, BLOCK_PREFIX_NAME);

impl KeyCodec<BlockInner> for HashValue {
    fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<BlockInner> for Block {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}
