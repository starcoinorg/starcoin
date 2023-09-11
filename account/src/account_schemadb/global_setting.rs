use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_schemadb::{
    define_schema,
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

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GlobalSettingKey {
    DefaultAddress,
    /// FIXME: once db support iter, remove this.
    AllAddresses,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalValue {
    pub(crate) addresses: Vec<AccountAddress>,
}

impl KeyCodec<GlobalSetting> for GlobalSettingKey {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        GlobalSettingKey::decode(data)
    }
}

impl ValueCodec<GlobalSetting> for GlobalValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.addresses.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        <Vec<AccountAddress>>::decode(data).map(|addresses| GlobalValue { addresses })
    }
}
