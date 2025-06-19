// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::legacy_state_migration::{
    check_legecy_data_has_migration, maybe_legacy_account_state_migration,
};
use starcoin_genesis::Genesis;
use starcoin_statedb::ChainStateDB;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::state_view::StateReaderExt;

#[test]
pub fn test_legacy_account_state_migration() -> anyhow::Result<()> {
    starcoin_logger::init_for_test();

    let (storage, _, chain_info, _) =
        Genesis::init_storage_for_test(&ChainNetwork::new_builtin(BuiltinNetworkID::Test))?;

    // Execute the migration
    maybe_legacy_account_state_migration(
        storage.clone(),
        Some(chain_info.status().head.state_root()),
        Some(50),
    )?;

    let statedb = ChainStateDB::new(storage.clone(), None);

    // Verify 0xdb2ba632664e1579e6bd949c538405c2 account balance
    let account1 = AccountAddress::from_hex_literal("0xdb2ba632664e1579e6bd949c538405c2")?;
    assert_eq!(statedb.get_balance(account1)?.unwrap_or(0), 24453);

    // Verify version is 12
    assert!(check_legecy_data_has_migration(&statedb)?);

    Ok(())
}
