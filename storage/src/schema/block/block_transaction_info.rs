use crate::BLOCK_TRANSACTION_INFOS_PREFIX_NAME;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};

define_schema!(
    BlockTransactionInfo,
    HashValue,
    Vec<HashValue>,
    BLOCK_TRANSACTION_INFOS_PREFIX_NAME
);

impl KeyCodec<BlockTransactionInfo> for HashValue {
    fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<BlockTransactionInfo> for Vec<HashValue> {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}
