// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! define_storage {
    ($storage_type: ident, $key_type: ty, $value_type: ty, $prefix_name: expr) => {
        #[derive(Clone)]
        pub struct $storage_type {
            store: $crate::storage::InnerStorage<Self>,
        }

        impl $storage_type {
            pub fn new(instance: $crate::storage::StorageInstance) -> Self {
                Self {
                    store: $crate::storage::InnerStorage::new(instance),
                }
            }
        }

        impl $crate::storage::ColumnFamily for $storage_type {
            type Key = $key_type;
            type Value = $value_type;

            fn name() -> $crate::storage::ColumnFamilyName {
                $prefix_name
            }
        }

        impl $crate::storage::SchemaStorage for $storage_type {
            fn get_store(&self) -> &$crate::storage::InnerStorage<Self> {
                &self.store
            }
        }
    };
}
