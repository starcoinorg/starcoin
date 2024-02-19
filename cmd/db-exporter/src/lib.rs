// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::sync::Arc;
use starcoin_storage::{
    db_storage::{
        DBStorage
    },
    Storage,
    StorageVersion,
    storage::StorageInstance,
    cache_storage::CacheStorage
};
use anyhow::Result;

pub mod command_decode_payload;
pub mod command_progress;
pub mod verify_header;
pub mod verify_module;


pub fn init_db_obj(
    db_path: PathBuf,
) -> Result<Arc<Storage>> {
    let db_storage = DBStorage::open_with_cfs(
        db_path.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    Ok(Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?))
}