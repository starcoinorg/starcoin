use super::AccountAddressWrapper;
use starcoin_account_api::Setting;
use starcoin_schemadb::{
    define_schema,
    error::{StoreError, StoreResult},
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
    fn encode_key(&self) -> StoreResult<Vec<u8>> {
        Ok(self.0.to_vec())
    }

    fn decode_key(data: &[u8]) -> StoreResult<Self> {
        AccountAddressWrapper::try_from(data).map_err(StoreError::DecodeError)
    }
}
/// Setting use json encode/decode for support more setting field in the future.
impl ValueCodec<AccountSetting> for SettingWrapper {
    fn encode_value(&self) -> StoreResult<Vec<u8>> {
        Ok(serde_json::to_vec(&self.0).map_err(anyhow::Error::new)?)
    }

    fn decode_value(data: &[u8]) -> StoreResult<Self> {
        Ok(SettingWrapper(
            serde_json::from_slice(data).map_err(anyhow::Error::new)?,
        ))
    }
}
