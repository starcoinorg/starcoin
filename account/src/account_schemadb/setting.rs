use super::AccountAddressWrapper;
use anyhow::Result;
use starcoin_account_api::Setting;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};

pub const SETTING_PREFIX_NAME: ColumnFamilyName = "account_settings";

define_schema!(
    AccountSetting,
    AccountAddressWrapper,
    SettingWrapper,
    SETTING_PREFIX_NAME
);

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SettingWrapper(pub(crate) Setting);
impl From<Setting> for SettingWrapper {
    fn from(setting: Setting) -> Self {
        Self(setting)
    }
}

impl KeyCodec<AccountSetting> for AccountAddressWrapper {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.0.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        AccountAddressWrapper::try_from(data)
    }
}
/// Setting use json encode/decode for support more setting field in the future.
impl ValueCodec<AccountSetting> for SettingWrapper {
    fn encode_value(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.0).map_err(Into::into)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        serde_json::from_slice::<Setting>(data)
            .map(Into::into)
            .map_err(Into::into)
    }
}
