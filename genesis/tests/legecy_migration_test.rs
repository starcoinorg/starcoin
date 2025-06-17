use starcoin_genesis::legecy_state_migration::legecy_account_state_migration;
use starcoin_state_api::ChainStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    db_storage::DBStorage,
    storage::{CodecKVStore, StorageInstance},
};
use starcoin_storage::{Storage, StorageVersion};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::AccountResource;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::StructTag;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::BalanceResource;
use starcoin_vm_types::state_view::StateReaderExt;
use std::env::temp_dir;
use std::sync::Arc;

#[test]
pub fn test_legecy_account_state_migration() -> anyhow::Result<()> {
    // Create a temporary directory for test storage
    let temp_dir = starcoin_config::temp_dir();

    let db_storage = DBStorage::open_with_cfs(
        temp_dir,
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
    legecy_account_state_migration(&statedb)?;

    // Verify 0x1 account balance
    let account1 = AccountAddress::from_hex_literal("0x1")?;
    assert!(statedb.get_balance(account1)?.unwrap_or(0) > 0);
    Ok(())
}
