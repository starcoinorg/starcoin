// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::legacy_state_migration::{
    check_legecy_data_has_migration, maybe_legacy_account_state_migration_with_statedb,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::state_view::StateReaderExt;

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
