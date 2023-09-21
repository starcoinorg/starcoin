// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::schema::table_info::TableInfoSchema;
use anyhow::Result;
use starcoin_schemadb::{db::DBStorage, SchemaBatch};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct TableInfoStorage {
    db: Arc<DBStorage>,
}

impl TableInfoStorage {
    pub(crate) fn new(db: &Arc<DBStorage>) -> Self {
        Self { db: Arc::clone(db) }
    }
}

pub trait TableInfoStore {
    fn get_table_info(&self, key: &TableHandle) -> Result<Option<TableInfo>>;
    fn save_table_info(&self, key: &TableHandle, table_info: &TableInfo) -> Result<()>;
    fn get_table_infos(&self, keys: &[TableHandle]) -> Result<Vec<Option<TableInfo>>>;
    fn save_table_infos(&self, table_infos: &[(TableHandle, TableInfo)]) -> Result<()>;
}

impl TableInfoStore for TableInfoStorage {
    fn get_table_info(&self, key: &TableHandle) -> Result<Option<TableInfo>> {
        self.db.get::<TableInfoSchema>(key)
    }

    fn save_table_info(&self, key: &TableHandle, table_info: &TableInfo) -> Result<()> {
        self.db.put::<TableInfoSchema>(key, table_info)
    }

    fn get_table_infos(&self, keys: &[TableHandle]) -> Result<Vec<Option<TableInfo>>> {
        self.db.batched_multi_get::<TableInfoSchema>(keys)
    }

    fn save_table_infos(&self, table_infos: &[(TableHandle, TableInfo)]) -> Result<()> {
        let batch = SchemaBatch::new();
        for info in table_infos {
            batch.put::<TableInfoSchema>(&info.0, &info.1)?;
        }
        self.db.write_schemas(batch)
    }
}
