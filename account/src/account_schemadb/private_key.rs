use super::AccountAddressWrapper;
use anyhow::Result;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};

pub const ENCRYPTED_PRIVATE_KEY_PREFIX_NAME: ColumnFamilyName = "encrypted_private_key";

define_schema!(
    PrivateKey,
    AccountAddressWrapper,
    EncryptedPrivateKey,
    ENCRYPTED_PRIVATE_KEY_PREFIX_NAME
);

impl KeyCodec<PrivateKey> for AccountAddressWrapper {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.0.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        AccountAddressWrapper::try_from(data)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EncryptedPrivateKey(pub Vec<u8>);
impl From<Vec<u8>> for EncryptedPrivateKey {
    fn from(s: Vec<u8>) -> Self {
        Self(s)
    }
}

impl ValueCodec<PrivateKey> for EncryptedPrivateKey {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.0.clone())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(EncryptedPrivateKey(data.to_vec()))
    }
}
