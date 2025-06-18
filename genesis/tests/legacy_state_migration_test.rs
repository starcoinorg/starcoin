// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_genesis::legacy_state_migration::legacy_account_state_migration;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::state_view::StateReaderExt;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
pub fn test_legacy_account_state_migration() -> anyhow::Result<()> {
    starcoin_logger::init_for_test();

    // Create a temporary directory for test storage
    let temp_dir = tempdir()?;

    let db_storage = DBStorage::open_with_cfs(
        temp_dir.path(),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        false,
        Default::default(),
        None,
    )?;
    let statedb = ChainStateDB::new(
        Arc::new(Storage::new(StorageInstance::new_db_instance(db_storage))?),
        None,
    );

    // Execute the migration
    legacy_account_state_migration(&statedb, Some(50))?;

    // Verify 0x1 account balance
    let account1 = AccountAddress::from_hex_literal("0xdb2ba632664e1579e6bd949c538405c2")?;
    assert_eq!(statedb.get_balance(account1)?.unwrap_or(0), 24453);
    Ok(())
}
