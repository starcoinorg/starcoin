// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::ChainNetwork;
use starcoin_genesis::legacy_state_migration::{
    check_legecy_data_has_migration, maybe_legacy_account_state_migration_with_statedb,
};
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::StructTag,
    state_set::{AccountStateSet, ChainStateSet, StateSet},
};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::on_chain_config::OnChainConfig;
use starcoin_vm_types::{
    account_config::BalanceResource, on_chain_config, on_chain_config::Version,
    state_view::StateReaderExt,
};
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

    let (statedb, _net) = test_helper::executor::prepare_genesis();

    // Check 0xdb2ba632664e1579e6bd949c538405c2 is 0
    // assert_eq!(
    //     statedb
    //         .get_balance(AccountAddress::from_hex_literal(
    //             "0xdb2ba632664e1579e6bd949c538405c2"
    //         )?)?
    //         .unwrap_or(0),
    //     0
    // );

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

#[test]
pub fn test_legacy_account_state_migration_only_for_0x3dd0a058b690062d915163078606e0d5(
) -> anyhow::Result<()> {
    starcoin_logger::init_for_test();

    let csv_content = std::fs::read_to_string(
        "migration/legecy-state-data-for-0x3dd0a058b690062d915163078606e0d5.csv",
    )?;
    let mut csv_reader = csv::Reader::from_reader(csv_content.as_bytes());

    let mut account_states = Vec::new();
    let mut replace_balance = 0;
    let mut target_addr: AccountAddress = AccountAddress::random();

    for result in csv_reader.records() {
        let record = result.expect("Failed to read CSV record");
        let address_str = &record[0];

        target_addr = serde_json::from_str(address_str)?;
        info!(
            "test_read_0x1_balance_from_csv | address str: {}, addr hex literal: {}",
            address_str,
            target_addr.to_hex_literal()
        );

        // Deserialize resource_state_set
        let resource_state_set: StateSet = serde_json::from_str(&record[4])?;
        // let code_state_set: StateSet = serde_json::from_str(&record[2])?;

        let writeable_account_state_set = AccountStateSet::new(vec![
            // Some(code_state_set.clone()),
            None,
            Some(resource_state_set.clone()),
        ]);

        // Check the legitimacy and integrity of the data
        for (struct_tag_bcs, blob_bcs) in resource_state_set.iter() {
            let struct_tag: StructTag = bcs_ext::from_bytes(struct_tag_bcs)?;
            // println!("struct_tag: {:?}, blob: {:?}", struct_tag, blob_bcs);
            if struct_tag.module == Identifier::new("Account")?
                && struct_tag.name == Identifier::new("Balance")?
            {
                let balance = bcs_ext::from_bytes::<BalanceResource>(blob_bcs)?.token();
                info!("test_legacy_account_state_migration_only_for_0x3dd0a058b690062d915163078606e0d5 | balance: {:?}", balance);
                assert_eq!(balance, 3890799);
                replace_balance = balance;
                break;
            }
        }

        // Add to account states for later application
        account_states.push((target_addr, writeable_account_state_set));
    }

    // Apply the verified data to statedb
    let (statedb, _network) = test_helper::executor::prepare_genesis();
    let before_state_root = statedb.state_root();
    info!("before_state_root: {:?}", before_state_root);
    if !account_states.is_empty() {
        statedb.apply(ChainStateSet::new(account_states))?;
    }
    statedb.commit()?;
    statedb.flush()?;

    let end_state_root = statedb.state_root();
    info!("end_state_root: {:?}", end_state_root);
    assert_ne!(before_state_root, end_state_root);

    let balance = statedb.get_balance(target_addr)?;
    assert_eq!(balance.unwrap_or(0), replace_balance);

    Ok(())
}

/// Check the legitimacy and integrity of the data in csv
#[test]
pub fn test_read_0x1_balance_from_csv() -> anyhow::Result<()> {
    starcoin_logger::init_for_test();

    let csv_content = std::fs::read_to_string("migration/legecy-state-data-for-0x1.csv")?;
    let mut csv_reader = csv::Reader::from_reader(csv_content.as_bytes());

    let mut account_states = Vec::new();
    let mut replace_balance = 0;
    let mut replaced_version = 0;

    let on_chain_version_struct_tag = on_chain_config::access_path_for_config(
        genesis_address(),
        Identifier::new("Version")?,
        Identifier::new("Version")?,
        vec![],
    ).path.as_struct_tag().unwrap().clone();
    info!(
        "test_read_0x1_balance_from_csv | version struct tag: {:?}",
        on_chain_version_struct_tag
    );

    for result in csv_reader.records() {
        let record = result.expect("Failed to read CSV record");
        let address_str = &record[0];

        let addr: AccountAddress = serde_json::from_str(address_str)?;
        let address_hex_literal = addr.to_hex_literal();
        info!(
            "test_read_0x1_balance_from_csv | address str: {}, addr hex literal: {}",
            address_str, address_hex_literal
        );

        if addr != AccountAddress::ONE {
            continue;
        }

        // Deserialize resource_state_set
        let resource_state_set: StateSet = serde_json::from_str(&record[4])?;
        let code_state_set: StateSet = serde_json::from_str(&record[2])?;

        let writeable_account_state_set = AccountStateSet::new(vec![
            Some(code_state_set.clone()),
            Some(resource_state_set.clone()),
        ]);

        // Check the legitimacy and integrity of the data
        for (struct_tag_bcs, blob_bcs) in resource_state_set.iter() {
            let struct_tag: StructTag = bcs_ext::from_bytes(struct_tag_bcs)?;
            info!("struct_tag: {:?}", struct_tag);
            if struct_tag.module == Identifier::new("Account")?
                && struct_tag.name == Identifier::new("Balance")?
            {
                let balance = bcs_ext::from_bytes::<BalanceResource>(blob_bcs)?;
                info!("test_read_0x1_balance_from_csv | balance: {:?}", balance);
                replace_balance = balance.token();
                assert_eq!(replace_balance, 10000);
                continue;
            }

            if struct_tag == on_chain_version_struct_tag {
                let version = bcs_ext::from_bytes::<Version>(blob_bcs)?;
                info!("version: {:?}", version);
                replaced_version = version.major;
                assert_eq!(replaced_version, 12);
                continue;
            }
        }

        // Add to account states for later application
        account_states.push((AccountAddress::ONE, writeable_account_state_set));
    }

    // Apply the verified data to statedb
    let (statedb, _network) = test_helper::executor::prepare_genesis();
    let before_state_root = statedb.state_root();
    info!("before_state_root: {:?}", before_state_root);
    if !account_states.is_empty() {
        statedb.apply(ChainStateSet::new(account_states))?;
    }
    statedb.commit()?;
    statedb.flush()?;

    let end_state_root = statedb.state_root();
    info!("end_state_root: {:?}", end_state_root);

    assert_ne!(before_state_root, end_state_root);

    let version = statedb.get_on_chain_config::<Version>()?;
    assert_eq!(
        version.unwrap_or(Version { major: 0 }).major,
        replaced_version
    );

    let balance = statedb.get_balance(AccountAddress::ONE)?;
    assert_eq!(balance.unwrap_or(0), replace_balance);

    Ok(())
}
