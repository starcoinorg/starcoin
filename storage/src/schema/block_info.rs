use crate::BLOCK_INFO_PREFIX_NAME;
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_types::block::BlockInfo as BlockInfoType;

define_schema!(BlockInfo, HashValue, BlockInfoType, BLOCK_INFO_PREFIX_NAME);

impl KeyCodec<BlockInfo> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<BlockInfo> for BlockInfoType {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
