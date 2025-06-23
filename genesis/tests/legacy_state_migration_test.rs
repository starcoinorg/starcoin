// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::legacy_state_migration::{
    check_legecy_data_has_migration, maybe_legacy_account_state_migration_with_statedb,
};
use starcoin_genesis::Genesis;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::{
    account_address::AccountAddress,
    state_set::ChainStateSet,
    state_set::{AccountStateSet, StateSet},
};
use starcoin_vm_types::state_view::StateReaderExt;
use std::fs::create_dir_all;
use std::sync::Arc;
use tempfile::TempDir;

/// Create a ChainStateDB with real storage from a test directory with custom options
///
/// # Arguments
/// * `db_name` - Optional custom name for the database directory (default: "test_db")
///
/// # Returns
/// * `ChainStateDB` - The initialized state database
/// * `ChainNetwork` - The test network configuration
/// * `TempDir` - The temporary directory containing the database (will be auto-cleaned)
fn create_test_statedb_with_genesis_custom(
    db_name: Option<&str>,
) -> anyhow::Result<(ChainStateDB, ChainNetwork, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_name = db_name.unwrap_or("test_db");
    let test_db_path = temp_dir.path().join(db_name);
    if !test_db_path.exists() {
        create_dir_all(&test_db_path)?;
    }

    // Create real storage and statedb
    let db_storage = DBStorage::open_with_cfs(
        &test_db_path,
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

    // Build and execute genesis
    let net = ChainNetwork::new_test();
    let genesis_txn = Genesis::build_genesis_transaction(&net)?;
    Genesis::execute_genesis_txn(&statedb, genesis_txn)?;

    Ok((statedb, net, temp_dir))
}

#[test]
pub fn test_legacy_account_state_migration_only_for_0x1() -> anyhow::Result<()> {
    starcoin_logger::init_for_test();

    // let statedb = test_helper::executor::prepare_customized_genesis(&ChainNetwork::new_builtin(
    //     BuiltinNetworkID::Main,
    // ));

    let (statedb, _, _) = create_test_statedb_with_genesis_custom(Some("testdb"))?;
    let csv_content = std::fs::read_to_string("migration/legecy-state-data-for-0x1.csv")?;
    let mut csv_reader = csv::Reader::from_reader(csv_content.as_bytes());

    // Skip header and get the first data record
    let mut records = csv_reader.records();
    let record = records
        .next()
        .ok_or_else(|| anyhow::anyhow!("No records found in CSV"))??;

    let account_address: AccountAddress = serde_json::from_str(record.get(0).unwrap())?;
    let code_state = serde_json::from_str::<StateSet>(record.get(2).unwrap())?;
    let resource_state = serde_json::from_str::<StateSet>(record.get(4).unwrap())?;

    statedb.apply(ChainStateSet::new(vec![(
        account_address,
        AccountStateSet::new(vec![Some(code_state), Some(resource_state)]),
    )]))?;
    statedb.commit()?;
    statedb.flush()?;

    let account1 = AccountAddress::from_hex_literal("0x1")?;
    assert_eq!(statedb.get_balance(account1)?.unwrap_or(0), 10000);

    Ok(())
}

#[test]
pub fn test_legacy_account_state_migration() -> anyhow::Result<()> {
    starcoin_logger::init_for_test();

    let statedb = test_helper::executor::prepare_customized_genesis(&ChainNetwork::new_builtin(
        BuiltinNetworkID::Main,
    ));

    // Execute the migration
    maybe_legacy_account_state_migration_with_statedb(
        &statedb,
        Some(vec![
            AccountAddress::from_hex_literal("0x1")?,
            AccountAddress::from_hex_literal("0xdb2ba632664e1579e6bd949c538405c2")?,
        ]),
    )?;

    let account1 = AccountAddress::from_hex_literal("0x1")?;
    assert_eq!(statedb.get_balance(account1)?.unwrap_or(0), 10000);

    // Verify 0xdb2ba632664e1579e6bd949c538405c2 account balance
    let account2 = AccountAddress::from_hex_literal("0xdb2ba632664e1579e6bd949c538405c2")?;
    assert_eq!(statedb.get_balance(account2)?.unwrap_or(0), 24453);

    // Verify version is 12
    assert!(check_legecy_data_has_migration(&statedb)?);

    Ok(())
}
