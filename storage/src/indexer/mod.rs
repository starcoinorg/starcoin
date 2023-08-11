// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Store;
use anyhow::{bail, ensure, Result};
use starcoin_resource_viewer::{AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_vm_types::access_path::DataPath;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::{
    account_address::AccountAddress,
    identifier::ident_str,
    language_storage::StructTag,
    state_store::{
        state_key::StateKey,
        table::{TableHandle, TableInfo},
    },
    state_view::StateView,
    write_set::{WriteOp, WriteSet},
};
use std::collections::HashMap;

struct TableInfoParser<'a> {
    store: &'a dyn Store,
    annotator: &'a MoveValueAnnotator<'a>,
    result: HashMap<TableHandle, TableInfo>,
    pending_on: HashMap<TableHandle, Vec<&'a [u8]>>,
}

impl<'a> TableInfoParser<'a> {
    pub fn new(store: &'a dyn Store, annotator: &'a MoveValueAnnotator) -> Self {
        Self {
            store,
            annotator,
            result: HashMap::new(),
            pending_on: HashMap::new(),
        }
    }

    pub fn parse_write_op(&mut self, state_key: &'a StateKey, write_op: &'a WriteOp) -> Result<()> {
        use StateKey::*;

        if let Some(bytes) = write_op.bytes() {
            match state_key {
                AccessPath(access_path) => match &access_path.path {
                    DataPath::Code(_) => (),
                    DataPath::Resource(struct_tag) => {
                        self.parse_struct(struct_tag.clone(), bytes)?
                    }
                },
                TableItem(item) => self.parse_table_item(item.handle, bytes)?,
            }
        }
        Ok(())
    }

    fn parse_struct(&mut self, struct_tag: StructTag, bytes: &[u8]) -> Result<()> {
        self.parse_move_value(
            &self
                .annotator
                .view_value(&TypeTag::Struct(Box::new(struct_tag)), bytes)?,
        )
    }

    fn parse_table_item(&mut self, handle: TableHandle, bytes: &'a [u8]) -> Result<()> {
        match self.get_table_info(handle)? {
            Some(table_info) => {
                self.parse_move_value(&self.annotator.view_value(&table_info.value_type, bytes)?)?;
            }
            None => {
                self.pending_on
                    .entry(handle)
                    .or_insert_with(Vec::new)
                    .push(bytes);
            }
        }
        Ok(())
    }

    fn parse_move_value(&mut self, move_value: &AnnotatedMoveValue) -> Result<()> {
        match move_value {
            // Fixme, see the definition of AnnotatedMoveValue
            AnnotatedMoveValue::Vector(items) => {
                for item in items {
                    self.parse_move_value(item)?;
                }
            }
            AnnotatedMoveValue::Struct(struct_value) => {
                let struct_tag = &struct_value.type_;
                if Self::is_table(struct_tag) {
                    assert_eq!(struct_tag.type_params.len(), 2);
                    let table_info = TableInfo {
                        key_type: struct_tag.type_params[0].clone(),
                        value_type: struct_tag.type_params[1].clone(),
                    };
                    let table_handle = match &struct_value.value[0] {
                        (name, AnnotatedMoveValue::Address(handle)) => {
                            assert_eq!(name.as_ref(), ident_str!("handle"));
                            TableHandle(*handle)
                        }
                        _ => bail!("Table struct malformed. {:?}", struct_value),
                    };
                    self.save_table_info(table_handle, table_info)?;
                } else {
                    for (_identifier, field) in &struct_value.value {
                        self.parse_move_value(field)?;
                    }
                }
            }

            // there won't be tables in primitives
            AnnotatedMoveValue::U8(_) => {}
            AnnotatedMoveValue::U16(_) => {}
            AnnotatedMoveValue::U32(_) => {}
            AnnotatedMoveValue::U64(_) => {}
            AnnotatedMoveValue::U128(_) => {}
            AnnotatedMoveValue::U256(_) => {}
            AnnotatedMoveValue::Bool(_) => {}
            AnnotatedMoveValue::Address(_) => {}
            AnnotatedMoveValue::Bytes(_) => {}
        }
        Ok(())
    }

    fn save_table_info(&mut self, handle: TableHandle, info: TableInfo) -> Result<()> {
        if self.get_table_info(handle)?.is_none() {
            self.result.insert(handle, info);
            if let Some(pending_items) = self.pending_on.remove(&handle) {
                for bytes in pending_items {
                    self.parse_table_item(handle, bytes)?;
                }
            }
        }
        Ok(())
    }

    fn is_table(struct_tag: &StructTag) -> bool {
        struct_tag.address == AccountAddress::ONE
            && struct_tag.module.as_ident_str() == ident_str!("table")
            && struct_tag.name.as_ident_str() == ident_str!("Table")
    }

    fn get_table_info(&self, handle: TableHandle) -> Result<Option<TableInfo>> {
        match self.result.get(&handle) {
            Some(table_info) => Ok(Some(table_info.clone())),
            None => self.store.get_table_info(handle),
        }
    }

    pub fn finish(self, keys: &mut Vec<TableHandle>, values: &mut Vec<TableInfo>) -> Result<()> {
        ensure!(
            self.pending_on.is_empty(),
            "There is still pending table items to parse due to unknown table info for table handles: {:?}",
            self.pending_on.keys(),
        );

        self.result
            .into_iter()
            .for_each(|(table_handle, table_info)| {
                keys.push(table_handle);
                values.push(table_info);
            });

        Ok(())
    }
}

pub struct Indexer<'a> {
    db: &'a dyn Store,
}

impl<'a> Indexer<'a> {
    pub fn new(db: &'a dyn Store) -> Self {
        Self { db }
    }

    pub fn index(
        &self,
        state_view: &dyn StateView,
        write_sets: &Vec<WriteSet>,
        keys: &mut Vec<TableHandle>,
        values: &mut Vec<TableInfo>,
    ) -> Result<()> {
        let annotator = MoveValueAnnotator::new(state_view);
        let mut table_info_parser = TableInfoParser::new(self.db, &annotator);

        for write_set in write_sets {
            for (state_key, write_op) in write_set.iter() {
                table_info_parser.parse_write_op(state_key, write_op)?;
            }
        }

        table_info_parser.finish(keys, values)?;

        Ok(())
    }
}
