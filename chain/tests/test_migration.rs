// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod migration_tests {
    use log::debug;
    use starcoin_chain::ChainReader;
    use starcoin_config::{BuiltinNetworkID, ChainNetwork, DEFAULT_CACHE_SIZE};
    use starcoin_crypto::HashValue;
    use starcoin_data_migration::{do_migration, filter_chain_state_set, migrate_main_data_to_statedb, migrate_test_data_to_statedb};
    use starcoin_state_api::{AccountStateReader, ChainStateReader, ChainStateWriter};
    use starcoin_statedb::ChainStateDB;
    use starcoin_storage::{storage::StorageInstance, Storage};
    use starcoin_types::account_address::AccountAddress;
    use starcoin_types::identifier::Identifier;
    use starcoin_types::state_set::ChainStateSet;
    use starcoin_vm_types::account_config::genesis_address;
    use starcoin_vm_types::on_chain_config::access_path_for_config;
    use starcoin_vm_types::{
        account_config::association_address, on_chain_config::Version, state_view::StateReaderExt,
    };
    use std::sync::Arc;
    use tempfile::TempDir;
    use test_helper::{
        create_block_with_transactions, print_account_resource_set, print_bcs_decoded_resources,
    };

    /// Set up test environment with STARCOIN_USE_TEST_MIGRATION environment variable
    fn setup_main_environment() {
        // Set the environment variable for all tests in this module
        std::env::set_var("STARCOIN_MIGRATION_DATASET", "main");
    }

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
    fn verify_migration_results(statedb: &ChainStateDB, expect_version: u64) -> anyhow::Result<()> {
        let stdlib_version = statedb
            .get_on_chain_config::<Version>()?
            .map(|version| version.major)
            .unwrap_or(0);
        assert_eq!(
            stdlib_version, expect_version,
            "stdlib version should be {:?} after migration",
            expect_version
        );
        Ok(())
    }

    /// Test function to demonstrate BCS resource printing
    #[stest::test]
    fn test_print_bcs_decoded_resources() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        // Use the test snapshot data
        let (file_name, data_hash, snapshot_pack) =
            starcoin_data_migration::MigrationDataSet::test().as_tuple()?;

        // Extract BCS content from tar.gz (same as migrate_legacy_state_data)
        let temp_dir = tempfile::TempDir::new()?;
        let tar_file = flate2::read::GzDecoder::new(snapshot_pack);
        let mut archive = tar::Archive::new(tar_file);
        archive.unpack(&temp_dir)?;

        let bcs_path = temp_dir.path().join(file_name);
        assert!(bcs_path.exists(), "{:?} does not exist", file_name);

        let bcs_content = std::fs::read(bcs_path)?;

        // Verify hash
        assert_eq!(
            HashValue::sha3_256_of(&bcs_content),
            data_hash,
            "Content hash should be the same"
        );

        // Print the decoded resources
        print_bcs_decoded_resources(bcs_content)?;

        Ok(())
    }

    #[stest::test]
    pub fn test_migration_with_genesis_storage() -> anyhow::Result<()> {
        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Proxima);
        let temp_dir = starcoin_config::temp_dir();
        let (_block_chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &net,
                temp_dir.path().to_path_buf(),
            )?;

        // Execute migration (this will verify file hash and basic functionality)
        do_migration(&statedb, net.chain_id())?;

        verify_migration_results(&statedb, 4)
    }

    /// Test basic migration functionality and file integrity
    #[stest::test]
    fn test_migration_basic_functionality_and_integrity() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Dev)?;

        // Execute migration (this will verify file hash and basic functionality)
        do_migration(&chain_state_db, net.chain_id())?;

        // Verify post-migration state
        let new_statedb = chain_state_db.fork_at(chain_state_db.state_root());

        verify_migration_results(&new_statedb, 4)?;

        Ok(())
    }

    /// Test migration with comparing two chains
    #[stest::test]
    fn test_migration_with_comparing_two_chains() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // First, execute migration to get the expected state
        do_migration(&chain_state_db, net.chain_id())?;

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
        do_migration(&chain_state_db, net.chain_id())?;

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
        verify_migration_results(&after_statedb, 4)?;

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
    #[stest::test]
    fn test_migration_idempotency() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // Execute migration again
        do_migration(&chain_state_db, net.chain_id())?;
        let first_root = chain_state_db.state_root();
        let first_statedb = chain_state_db.fork_at(first_root);

        let first_version = first_statedb
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        let first_balance = first_statedb.get_balance(AccountAddress::ONE)?.unwrap_or(0);

        // Second migration execution
        do_migration(&chain_state_db, net.chain_id())?;
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
        let (main_net, _storage_main, chain_state_db_main) =
            create_test_environment(BuiltinNetworkID::Main)?;
        do_migration(&chain_state_db_main, main_net.chain_id())?;

        // Test proxima network
        let (_proxima_net, _storage_proxima, chain_state_db_proxima) =
            create_test_environment(BuiltinNetworkID::Proxima)?;
        do_migration(&chain_state_db_proxima, main_net.chain_id())?;

        // Verify post-migration state for both networks
        let main_statedb = chain_state_db_main.fork_at(chain_state_db_main.state_root());
        let proxima_statedb = chain_state_db_proxima.fork_at(chain_state_db_proxima.state_root());

        verify_migration_results(&main_statedb, 4)?;
        verify_migration_results(&proxima_statedb, 4)?;

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

    /// Test migration with 0x1 account state data specifically
    #[stest::test(timeout = 5000)]
    fn test_migration_main_data() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let (net, _storage, chain_state_db) = create_test_environment(BuiltinNetworkID::Main)?;

        // First, execute migration to get the expected state
        setup_main_environment();
        do_migration(&chain_state_db, net.chain_id())?;
        let expected_state_root = chain_state_db.state_root();
        let expected_statedb = chain_state_db.fork_at(expected_state_root);

        // Export only the 0x1 account state data
        let version = expected_statedb
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        assert_eq!(version, 12, "0x1 account should exist after migration");

        Ok(())
    }

    /// Test block mining simulation with data migration in memory
    /// This test simulates the process of building a blockchain in memory,
    /// mining an empty block, and verifying data migration results
    #[stest::test(timeout = 6000)]
    fn test_block_migration_with_blockchain_mining() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        // Create a test network (using Main to trigger migration)
        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Main);
        let temp = TempDir::new()?;

        // Initialize blockchain with genesis in memory
        let (mut chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &net,
                temp.path().to_path_buf(),
            )?;

        // Get current header (genesis block)
        let genesis_header = chain.current_header();
        assert_eq!(
            genesis_header.number(),
            0,
            "Should start with genesis block"
        );

        const MAX_TEST_BLOCKS: usize = 4;

        // Create N blocks (empty block)
        let mut state_root = genesis_header.state_root().clone();
        for _ in 0..MAX_TEST_BLOCKS {
            // Create block template for the first block (block #1) - empty block
            let (_executed_block, _stateroot) =
                create_block_with_transactions(&mut chain, &net, association_address(), vec![])?;

            debug!(
                "test_block_migration_with_blockchain_mining | executed_block header stateroot:{:?}, state_root1: {:?}, chain id: {:?}",
                _executed_block.header().state_root(),
                _executed_block.multi_state().state_root1(),
                statedb.get_chain_id()?,
            );
            state_root = _stateroot;
        }

        let statedb = statedb.fork_at(state_root);
        assert!(
            statedb
                .get_account_state_set(&genesis_address())
                .unwrap()
                .unwrap()
                .resource_set()
                .unwrap()
                .len()
                > 400,
            "New genesis resource count should bigger than 400"
        );

        let version = statedb
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        let token_info = statedb.get_stc_info()?.unwrap();

        debug!(
            "The latest version number: {:?}, stc total value: {:?}",
            version, token_info.total_value
        );
        Ok(())
    }

    #[stest::test]
    pub fn test_write_version_to_db_to_find_key_hash() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        // Create a test network (using Main to trigger migration)
        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Proxima);
        let temp = TempDir::new()?;

        // Initialize blockchain with genesis in memory
        let (_chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &net,
                temp.path().to_path_buf(),
            )?;

        debug!("start set version");
        statedb.set(
            &access_path_for_config(
                genesis_address(),
                Identifier::new("Version").unwrap(),
                Identifier::new("Version").unwrap(),
                vec![],
            ),
            bcs_ext::to_bytes(&Version { major: 1000 })?,
        )?;
        statedb.commit()?;
        statedb.flush()?;

        debug!("start get version");
        let version = statedb
            .get_on_chain_config::<Version>()?
            .map(|v| v.major)
            .unwrap_or(0);
        assert_eq!(version, 1000);
        Ok(())
    }

    #[stest::test]
    pub fn test_filter_account_state_set_basic() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();
        log::set_max_level(log::LevelFilter::Debug);

        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Proxima);

        let (_chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb(&net, None)?;

        let address = AccountAddress::ONE;

        let account_state = statedb
            .get_account_state_set(&address)?
            .expect("get account state set should successfully");
        let chain_state_set = ChainStateSet::new(vec![(address, account_state)]);
        let filtered_chain_state_set = filter_chain_state_set(chain_state_set, &statedb)?;
        statedb.apply(filtered_chain_state_set)?;

        Ok(())
    }

    // TODO(BobOng):[migration] should open after apply 0x1 succeed
    // /// Test migration data verification during peer-to-peer block synchronization
    // /// This test simulates a scenario where:
    // /// 1. A source node has migration data applied
    // /// 2. A target node syncs blocks from the source node
    // /// 3. Verifies that the migrated state is correctly synchronized
    // #[stest::test(timeout = 10000)]
    // fn test_migration_verification_during_peer_sync() -> anyhow::Result<()> {
    //     starcoin_logger::init_for_test();
    //
    //     // Create source node with migration data
    //     let source_net = ChainNetwork::new_builtin(BuiltinNetworkID::Proxima);
    //     let source_temp = TempDir::new()?;
    //
    //     // Initialize source blockchain with genesis
    //     let (mut source_chain, source_statedb) =
    //         test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
    //             &source_net,
    //             source_temp.path().to_path_buf(),
    //         )?;
    //
    //     // Create a few blocks after migration to simulate real blockchain state
    //     for i in 0..3 {
    //         let (executed_block, _) = create_block_with_transactions(
    //             &mut source_chain,
    //             &source_net,
    //             association_address(),
    //             vec![],
    //         )?;
    //
    //         debug!(
    //             "Source node block {} created, number: {}, state root: {:?}",
    //             i,
    //             executed_block.block().header().number(),
    //             executed_block.block().header().state_root()
    //         );
    //     }
    //
    //     // Get source node's final state
    //     let source_head = source_chain.current_header();
    //     let source_state_root = source_head.state_root();
    //
    //     // Verify source node has migration data
    //     let source_version = source_statedb
    //         .get_on_chain_config::<Version>()?
    //         .map(|v| v.major)
    //         .unwrap_or(0);
    //     assert_eq!(
    //         source_version, 4,
    //         "Source node should have migrated version 4"
    //     );
    //
    //     // Create target node (empty, no migration data)
    //     let target_net = ChainNetwork::new_builtin(BuiltinNetworkID::Proxima);
    //     let target_temp = TempDir::new()?;
    //
    //     let (mut target_chain, target_statedb) =
    //         test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
    //             &target_net,
    //             target_temp.path().to_path_buf(),
    //         )?;
    //
    //     // Verify target node starts without migration data
    //     let target_version_before = target_statedb
    //         .get_on_chain_config::<Version>()?
    //         .map(|v| v.major)
    //         .unwrap_or(0);
    //     assert_eq!(
    //         target_version_before, 11,
    //         "Target node should start with version 11"
    //     );
    //
    //     // Simulate block synchronization from source to target
    //     // This mimics the sync process where target node receives and applies blocks from source
    //     // Skip genesis block (number=0) since target already has its own genesis
    //     let source_blocks =
    //         source_chain.get_blocks_by_number(Some(1), false, source_head.number() + 1)?;
    //
    //     debug!(
    //         "Syncing {} blocks from source (height {}) to target",
    //         source_blocks.len(),
    //         source_head.number()
    //     );
    //
    //     // Apply each block from source to target (simulating sync)
    //     for (i, block) in source_blocks.iter().enumerate() {
    //         debug!(
    //             "Applying source block {} to target: number={}, id={:?}",
    //             i,
    //             block.header().number(),
    //             block.id()
    //         );
    //
    //         // Apply block to target chain (simulating sync process)
    //         let apply_result = target_chain.apply(block.clone());
    //
    //         if let Err(e) = apply_result {
    //             // If this is a migration block (block #3), it should trigger migration
    //             if block.header().number() == 3 {
    //                 debug!("Migration block applied, checking state...");
    //
    //                 // Verify migration was triggered and applied
    //                 let target_version_after = target_statedb
    //                     .get_on_chain_config::<Version>()?
    //                     .map(|v| v.major)
    //                     .unwrap_or(0);
    //
    //                 assert_eq!(
    //                     target_version_after, 11,
    //                     "Target node should have version 11 after migration block"
    //                 );
    //
    //                 debug!("Migration verification passed on target node");
    //             } else {
    //                 return Err(e.into());
    //             }
    //         }
    //     }
    //
    //     // Verify final state consistency between source and target
    //     let target_head = target_chain.current_header();
    //     let target_state_root = target_head.state_root();
    //
    //     debug!(
    //         "Final verification - Source: number={}, root={:?}, Target: number={}, root={:?}",
    //         source_head.number(),
    //         source_state_root,
    //         target_head.number(),
    //         target_state_root
    //     );
    //
    //     // Verify block numbers match
    //     assert_eq!(
    //         source_head.number(),
    //         target_head.number(),
    //         "Source and target should have same block number after sync"
    //     );
    //
    //     // Verify state roots match (indicating same state)
    //     assert_eq!(
    //         source_state_root, target_state_root,
    //         "Source and target should have same state root after sync"
    //     );
    //
    //     // Verify target node has correct migration data
    //     let target_version_final = target_statedb
    //         .get_on_chain_config::<Version>()?
    //         .map(|v| v.major)
    //         .unwrap_or(0);
    //     assert_eq!(
    //         target_version_final, 11,
    //         "Target node should have correct migration version after sync"
    //     );
    //
    //     debug!("Peer sync migration verification completed successfully");
    //     Ok(())
    // }
}
