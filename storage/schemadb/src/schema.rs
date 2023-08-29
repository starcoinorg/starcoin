// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use core::hash::Hash;
use std::fmt::Debug;

pub trait KeyCodec<S: Schema + ?Sized>: Clone + Sized + Debug + Send + Sync {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_key(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_key(data: &[u8]) -> Result<Self>;
}

pub trait ValueCodec<S: Schema + ?Sized>: Clone + Sized + Debug + Send + Sync {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_value(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_value(data: &[u8]) -> Result<Self>;
}

pub trait SeekKeyCodec<S: Schema + ?Sized> {
    fn encode_seek_key(&self) -> Result<Vec<u8>>;
}

// auto implements for all keys
impl<S, K> SeekKeyCodec<S> for K
where
    S: Schema,
    K: KeyCodec<S>,
{
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        <K as KeyCodec<S>>::encode_key(self)
    }
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
