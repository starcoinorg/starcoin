use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_schemadb::{
    define_schema,
    error::{StoreError, StoreResult},
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};
use starcoin_types::account_address::AccountAddress;

pub const GLOBAL_PREFIX_NAME: ColumnFamilyName = "global";

define_schema!(
    GlobalSetting,
    GlobalSettingKey,
    GlobalValue,
    GLOBAL_PREFIX_NAME
);

#[derive(Default, Hash, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GlobalSettingKey {
    #[default]
    DefaultAddress,
    /// FIXME: once db support iter, remove this.
    AllAddresses,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalValue {
    pub(crate) addresses: Vec<AccountAddress>,
}

impl KeyCodec<GlobalSetting> for GlobalSettingKey {
    fn encode_key(&self) -> StoreResult<Vec<u8>> {
        self.encode()
            .map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_key(data: &[u8]) -> StoreResult<Self> {
        GlobalSettingKey::decode(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

impl ValueCodec<GlobalSetting> for GlobalValue {
    fn encode_value(&self) -> StoreResult<Vec<u8>> {
        self.addresses
            .encode()
            .map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> StoreResult<Self> {
        <Vec<AccountAddress>>::decode(data)
            .map(|addresses| GlobalValue { addresses })
            .map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
