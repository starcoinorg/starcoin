use crate::storage::{KeyCodec, ValueCodec};
use anyhow::Result;
use std::marker::PhantomData;

pub enum ScanDirection {
    Forward,
    Backward,
}

pub struct SchemaIterator<'a, K, V> {
    db_iter: rocksdb::DBRawIterator<'a>,
    direction: ScanDirection,
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

impl<'a, K, V> SchemaIterator<'a, K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    pub(crate) fn new(db_iter: rocksdb::DBRawIterator<'a>, direction: ScanDirection) -> Self {
        SchemaIterator {
            db_iter,
            direction,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
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
    pub fn seek(&mut self, seek_key: Vec<u8>) -> Result<()> {
        self.db_iter.seek(&seek_key);
        Ok(())
    }

    /// Seeks to the last key whose binary representation is less than or equal to that of the
    /// `seek_key`.
    pub fn seek_for_prev(&mut self, seek_key: Vec<u8>) -> Result<()> {
        self.db_iter.seek_for_prev(&seek_key);
        Ok(())
    }

    fn next_impl(&mut self) -> Result<Option<(K, V)>> {
        if !self.db_iter.valid() {
            self.db_iter.status()?;
            return Ok(None);
        }

        let raw_key = self.db_iter.key().expect("Iterator must be valid.");
        let raw_value = self.db_iter.value().expect("Iterator must be valid.");
        let key = K::decode_key(raw_key)?;
        let value = V::decode_value(raw_value)?;
        match self.direction {
            ScanDirection::Forward => self.db_iter.next(),
            ScanDirection::Backward => self.db_iter.prev(),
        }

        Ok(Some((key, value)))
    }
}

impl<'a, K, V> Iterator for SchemaIterator<'a, K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}
