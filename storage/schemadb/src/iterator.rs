// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::schema::{KeyCodec, Schema, SeekKeyCodec, ValueCodec};
use anyhow::Result;
use std::marker::PhantomData;

pub enum ScanDirection {
    Forward,
    Backward,
}

pub struct SchemaIterator<'a, S: Schema> {
    db_iter: rocksdb::DBRawIterator<'a>,
    direction: ScanDirection,
    phantom: PhantomData<S>,
}

impl<'a, S> SchemaIterator<'a, S>
where
    S: Schema,
{
    pub(crate) fn new(db_iter: rocksdb::DBRawIterator<'a>, direction: ScanDirection) -> Self {
        SchemaIterator {
            db_iter,
            direction,
            phantom: PhantomData,
        }
    }

    /// Seeks to the first key.
    pub fn seek_to_first(&mut self) {
        self.db_iter.seek_to_first();
    }

    /// Seeks to the last key.
    pub fn seek_to_last(&mut self) {
        self.db_iter.seek_to_last();
    }

    /// Seeks to the first key whose binary representation is equal to or greater than that of the
    /// `seek_key`.
    pub fn seek<SK: SeekKeyCodec<S>>(&mut self, seek_key: &SK) -> Result<()> {
        let raw_key = <SK as SeekKeyCodec<S>>::encode_seek_key(seek_key)?;
        self.db_iter.seek(&raw_key);
        Ok(())
    }

    /// Seeks to the last key whose binary representation is less than or equal to that of the
    /// `seek_key`.
    pub fn seek_for_prev<SK: SeekKeyCodec<S>>(&mut self, seek_key: &SK) -> Result<()> {
        let raw_key = <SK as SeekKeyCodec<S>>::encode_seek_key(seek_key)?;
        self.db_iter.seek_for_prev(&raw_key);
        Ok(())
    }

    fn next_impl(&mut self) -> Result<Option<(S::Key, S::Value)>> {
        if !self.db_iter.valid() {
            self.db_iter.status()?;
            return Ok(None);
        }

        let raw_key = self.db_iter.key().expect("Iterator must be valid.");
        let raw_value = self.db_iter.value().expect("Iterator must be valid.");
        let key = <S::Key as KeyCodec<S>>::decode_key(raw_key)?;
        let value = <S::Value as ValueCodec<S>>::decode_value(raw_value)?;
        match self.direction {
            ScanDirection::Forward => self.db_iter.next(),
            ScanDirection::Backward => self.db_iter.prev(),
        }

        Ok(Some((key, value)))
    }
}

impl<'a, S> Iterator for SchemaIterator<'a, S>
where
    S: Schema,
{
    type Item = Result<(S::Key, S::Value)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}
