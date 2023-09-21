use crate::TABLE_INFO_PREFIX_NAME;
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};

define_schema!(
    TableInfoSchema,
    TableHandle,
    TableInfo,
    TABLE_INFO_PREFIX_NAME
);

impl KeyCodec<TableInfoSchema> for TableHandle {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<TableInfoSchema> for TableInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
