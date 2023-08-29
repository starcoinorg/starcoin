use super::AccountAddressWrapper;
use anyhow::{format_err, Result};
use starcoin_account_api::AccountPublicKey;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};

pub const PUBLIC_KEY_PREFIX_NAME: ColumnFamilyName = "public_key";

define_schema!(
    PublicKey,
    AccountAddressWrapper,
    PublicKeyWrapper,
    PUBLIC_KEY_PREFIX_NAME
);

impl KeyCodec<PublicKey> for AccountAddressWrapper {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.0.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        AccountAddressWrapper::try_from(data)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PublicKeyWrapper(pub(crate) Option<AccountPublicKey>);
impl From<AccountPublicKey> for PublicKeyWrapper {
    fn from(s: AccountPublicKey) -> Self {
        Self(Some(s))
    }
}

impl From<PublicKeyWrapper> for AccountPublicKey {
    fn from(value: PublicKeyWrapper) -> Self {
        value.0.expect("NullValue")
    }
}

impl ValueCodec<PublicKey> for PublicKeyWrapper {
    fn encode_value(&self) -> Result<Vec<u8>> {
        match &self.0 {
            Some(p) => Ok(bcs_ext::to_bytes(&p)?),
            None => Err(format_err!("NullValue")),
        }
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(Self::from(bcs_ext::from_bytes::<AccountPublicKey>(data)?))
    }
}
