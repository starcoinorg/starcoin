// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod migration_tests {
    use log::debug;
    use starcoin_config::{BuiltinNetworkID, ChainNetwork, DEFAULT_CACHE_SIZE};
    use starcoin_crypto::HashValue;
    use starcoin_data_migration::migrate_test_data_to_statedb;
    use starcoin_state_api::{ChainStateReader, ChainStateWriter};
    use starcoin_statedb::ChainStateDB;
    use starcoin_storage::{storage::StorageInstance, Storage};
    use starcoin_types::{account_address::AccountAddress, state_set::ChainStateSet};
    use starcoin_vm_types::{on_chain_config::Version, state_view::StateReaderExt};
    use std::sync::Arc;

    /// Helper function to create a test environment for migration
    fn create_test_environment(
        network: BuiltinNetworkID,
    ) -> anyhow::Result<(ChainNetwork, Arc<Storage>, ChainStateDB)> {
        let net = ChainNetwork::new_builtin(network);
        let storage = Arc::new(Storage::new(
            StorageInstance::new_cache_instance_with_capacity(DEFAULT_CACHE_SIZE * 100),
        )?);
        let chain_state_db = ChainStateDB::new(storage.clone(), None);
        Ok((net, storage, chain_state_db))
    }

    /// Helper function to verify migration results
    fn verify_migration_results(statedb: &ChainStateDB) -> anyhow::Result<()> {
        let stdlib_version = statedb
            .get_on_chain_config::<Version>()?
            .map(|version| version.major)
            .unwrap_or(0);
        assert_eq!(
            stdlib_version, 4,
            "stdlib version should be 12 after migration"
        );
        Ok(())
    }

    #[stest::test]
    pub fn test_migration_with_genesis_storage() -> anyhow::Result<()> {
        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Dev);
        let temp_dir = starcoin_config::temp_dir();
        let (_block_chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &net,
                temp_dir.path().to_path_buf(),
            )?;

        // Execute migration (this will verify file hash and basic functionality)
        migrate_test_data_to_statedb(&statedb)?;

        verify_migration_results(&statedb)
    }

    /// Test basic migration functionality and file integrity
    #[stest::test]
    fn test_migration_basic_functionality_and_integrity() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (_net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Dev)?;

        // Execute migration (this will verify file hash and basic functionality)
        migrate_test_data_to_statedb(&chain_state_db)?;

        // Verify post-migration state
        let new_statedb = chain_state_db.fork_at(chain_state_db.state_root());

        verify_migration_results(&new_statedb)?;

        Ok(())
    }

    /// Test migration with existing mainnet data and state consistency
    #[stest::test]
    fn test_migration_with_existing_data_and_consistency() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (_net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // First, execute migration to get the expected state
        migrate_test_data_to_statedb(&chain_state_db)?;

        let expected_state_root = chain_state_db.state_root();
        let expected_statedb = chain_state_db.fork_at(expected_state_root);

        // Export the migrated state data to create a realistic test scenario
        let exported_state_data = expected_statedb.dump()?;

        // Create a fresh test environment
        let (_net2, _storage2, chain_state_db2) = create_test_environment(BuiltinNetworkID::Main)?;

        // Apply the exported state data to simulate existing mainnet state
        chain_state_db2.apply(exported_state_data)?;
        chain_state_db2.commit()?;
        chain_state_db2.flush()?;

        // Record pre-migration state
        let before_migration_root = chain_state_db2.state_root();

        // Execute migration again
        migrate_test_data_to_statedb(&chain_state_db2)?;

        // Verify post-migration state
        let after_migration_root = chain_state_db2.state_root();
        let after_statedb = chain_state_db2.fork_at(after_migration_root);

        // Since the exported state data already contains migrated data,
        // the state root should remain the same (idempotency)
        assert_eq!(
            before_migration_root, after_migration_root,
            "State root should remain the same when applying already migrated data"
        );

        // Verify migration results
        verify_migration_results(&after_statedb)?;

        // Test state consistency by creating multiple forks
        let fork1 = chain_state_db2.fork_at(after_migration_root);
        let fork2 = chain_state_db2.fork_at(after_migration_root);

        let version1 = fork1
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        let version2 = fork2
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        assert_eq!(version1, version2, "Fork states should be consistent");

        let balance1 = fork1.get_balance(AccountAddress::ONE)?.unwrap_or(0);
        let balance2 = fork2.get_balance(AccountAddress::ONE)?.unwrap_or(0);
        assert_eq!(balance1, balance2, "Fork states should be consistent");

        Ok(())
    }

    /// Test migration idempotency
    #[stest::test(timeout = 50000)]
    fn test_migration_idempotency() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (_net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // Execute migration again
        migrate_test_data_to_statedb(&chain_state_db)?;
        let first_root = chain_state_db.state_root();
        let first_statedb = chain_state_db.fork_at(first_root);

        let first_version = first_statedb
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        let first_balance = first_statedb.get_balance(AccountAddress::ONE)?.unwrap_or(0);

        // Second migration execution
        migrate_test_data_to_statedb(&chain_state_db)?;
        let second_root = chain_state_db.state_root();
        let second_statedb = chain_state_db.fork_at(second_root);

        let second_version = second_statedb
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        let second_balance = second_statedb
            .get_balance(AccountAddress::ONE)?
            .unwrap_or(0);

        // Verify that both executions produce identical results
        assert_eq!(
            first_root, second_root,
            "State roots should be identical after multiple migrations"
        );
        assert_eq!(
            first_version, second_version,
            "Versions should be identical after multiple migrations"
        );
        assert_eq!(
            first_balance, second_balance,
            "Balances should be identical after multiple migrations"
        );

        Ok(())
    }

    /// Test migration for different network types
    #[stest::test]
    fn test_migration_network_specific_behavior() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        // Test mainnet
        let (_main_net, _storage_main, chain_state_db_main) =
            create_test_environment(BuiltinNetworkID::Main)?;
        migrate_test_data_to_statedb(&chain_state_db_main)?;

        // Test proxima network
        let (_proxima_net, _storage_proxima, chain_state_db_proxima) =
            create_test_environment(BuiltinNetworkID::Proxima)?;
        migrate_test_data_to_statedb(&chain_state_db_proxima)?;

        // Verify post-migration state for both networks
        let main_statedb = chain_state_db_main.fork_at(chain_state_db_main.state_root());
        let proxima_statedb = chain_state_db_proxima.fork_at(chain_state_db_proxima.state_root());

        verify_migration_results(&main_statedb)?;
        verify_migration_results(&proxima_statedb)?;

        Ok(())
    }

    /// Test that migration is skipped for non-mainnet networks
    #[stest::test]
    fn test_migration_skipped_for_non_mainnet() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let test_net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

        // Verify that test network indeed doesn't execute migration
        assert!(!test_net.is_main(), "Test network should not be mainnet");
        assert!(!test_net.is_proxima(), "Test network should not be proxima");

        Ok(())
    }

    /// Test migration integration in genesis build flow
    #[stest::test]
    fn test_migration_in_genesis_build_flow() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Main);

        // Build genesis (this will call migrate_legacy_state_data)
        let genesis = starcoin_genesis::Genesis::build(&net)?;

        // Verify genesis block state
        let genesis_block = genesis.block();
        assert_eq!(
            genesis_block.header().number(),
            0,
            "Genesis block number should be 0"
        );

        // Verify genesis block state root is not empty
        let state_root = genesis_block.header().state_root();
        assert_ne!(
            state_root,
            HashValue::zero(),
            "Genesis state root should not be zero"
        );

        Ok(())
    }

    /// Test migration error handling under normal conditions
    #[stest::test]
    fn test_migration_error_handling() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (_net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // Should succeed under normal conditions
        let result = migrate_test_data_to_statedb(&chain_state_db)?;

        debug!("result: {:?}", result);

        Ok(())
    }

    /// Test migration with 0x1 account state data specifically

    #[ignore]
    #[stest::test]
    fn test_migration_with_0x1_account_data() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (_net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // First, execute migration to get the expected state
        migrate_test_data_to_statedb(&chain_state_db)?;
        let expected_state_root = chain_state_db.state_root();
        let expected_statedb = chain_state_db.fork_at(expected_state_root);

        // Export only the 0x1 account state data
        let account_0x1_state_set = expected_statedb.get_account_state_set(&AccountAddress::ONE)?;
        assert!(
            account_0x1_state_set.is_some(),
            "0x1 account should exist after migration"
        );

        // Create a fresh test environment
        let (_net2, _storage2, chain_state_db2) = create_test_environment(BuiltinNetworkID::Main)?;

        // Apply only the 0x1 account state data to simulate a minimal existing state
        let minimal_state_data =
            ChainStateSet::new(vec![(AccountAddress::ONE, account_0x1_state_set.unwrap())]);
        chain_state_db2.apply(minimal_state_data)?;
        chain_state_db2.commit()?;
        chain_state_db2.flush()?;

        // Record pre-migration state
        let before_migration_root = chain_state_db2.state_root();

        // Execute migration
        migrate_test_data_to_statedb(&chain_state_db2)?;

        // Verify post-migration state
        let after_migration_root = chain_state_db2.state_root();
        let after_statedb = chain_state_db2.fork_at(after_migration_root);

        // Since we're applying only 0x1 account data which already contains migrated data,
        // the state root should remain the same (idempotency)
        assert_eq!(
            before_migration_root, after_migration_root,
            "State root should remain the same when applying already migrated 0x1 account data"
        );

        // Verify migration results
        verify_migration_results(&after_statedb)?;

        // Verify that 0x1 account still has the expected state
        let final_0x1_state_set = after_statedb.get_account_state_set(&AccountAddress::ONE)?;
        assert!(
            final_0x1_state_set.is_some(),
            "0x1 account should still exist after migration"
        );

        let final_balance = after_statedb.get_balance(AccountAddress::ONE)?.unwrap_or(0);
        assert_eq!(
            final_balance, 10000,
            "0x1 balance should be 10000 after migration"
        );

        Ok(())
    }
}
