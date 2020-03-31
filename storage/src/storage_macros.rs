// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ColumnFamilyName;
#[macro_export]
macro_rules! define_storage {
    ($storage_type: ident, $key_type: ty, $value_type: ty, $cf_name: expr) => {
        pub(crate) struct $storage_type {
            store: CodecStorage<$key_type, $value_type>,
        }

        impl $storage_type {
            const COLUMN_FAMILY_NAME: $crate::ColumnFamilyName = $cf_name;
            pub fn new(kv_store: Arc<dyn Repository>) -> Self {
                Self {
                    store: CodecStorage::new(kv_store),
                }
            }
            pub fn save(&self, key: $key_type, value: $value_type) -> Result<()> {
                self.store.put(key, value)
            }

            pub fn get(&self, key: $key_type) -> Result<Option<$value_type>> {
                self.store.get(key)
            }

            pub fn remove(&self, key: $key_type) -> Result<()> {
                self.store.remove(key)
            }
        }
    };
}
