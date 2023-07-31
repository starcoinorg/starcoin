use super::error::StoreError;
use core::hash::Hash;
use std::fmt::Debug;
use std::result::Result;

pub trait KeyCodec<S: Schema + ?Sized>: Clone + Sized + Debug + Send + Sync {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_key(&self) -> Result<Vec<u8>, StoreError>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_key(data: &[u8]) -> Result<Self, StoreError>;
}

pub trait ValueCodec<S: Schema + ?Sized>: Clone + Sized + Debug + Send + Sync {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_value(&self) -> Result<Vec<u8>, StoreError>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_value(data: &[u8]) -> Result<Self, StoreError>;
}

pub trait Schema: Debug + Send + Sync + 'static {
    const COLUMN_FAMILY: &'static str;

    type Key: KeyCodec<Self> + Hash + Eq + Default;
    type Value: ValueCodec<Self> + Default + Clone;
}

#[macro_export]
macro_rules! define_schema {
    ($schema_type: ident, $key_type: ty, $value_type: ty, $cf_name: expr) => {
        #[derive(Clone, Debug)]
        pub(crate) struct $schema_type;

        impl $crate::schema::Schema for $schema_type {
            type Key = $key_type;
            type Value = $value_type;

            const COLUMN_FAMILY: &'static str = $cf_name;
        }
    };
}
