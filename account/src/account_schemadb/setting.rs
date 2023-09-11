use anyhow::Result;
use starcoin_account_api::Setting;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};
use starcoin_types::account_address::AccountAddress;

pub const SETTING_PREFIX_NAME: ColumnFamilyName = "account_settings";

define_schema!(AccountSetting, AccountAddress, Setting, SETTING_PREFIX_NAME);

impl KeyCodec<AccountSetting> for AccountAddress {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        AccountAddress::try_from(data).map_err(Into::into)
    }
}
/// Setting use json encode/decode for support more setting field in the future.
impl ValueCodec<AccountSetting> for Setting {
    fn encode_value(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(Into::into)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        serde_json::from_slice::<Setting>(data).map_err(Into::into)
    }
}
