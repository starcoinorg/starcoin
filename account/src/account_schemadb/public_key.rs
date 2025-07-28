use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_account_api::AccountPublicKey;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};
use starcoin_types::account_address::AccountAddress;

pub const PUBLIC_KEY_PREFIX_NAME: ColumnFamilyName = "public_key";

define_schema!(
    PublicKey,
    AccountAddress,
    AccountPublicKey,
    PUBLIC_KEY_PREFIX_NAME
);

impl KeyCodec<PublicKey> for AccountAddress {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        AccountAddress::try_from(data).map_err(Into::into)
    }
}

impl ValueCodec<PublicKey> for AccountPublicKey {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes::<AccountPublicKey>(data)
    }
}
