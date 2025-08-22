// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Temporarily disable migration tests
#![allow(unexpected_cfgs)]

#[cfg(test)]
#[cfg(feature = "migration_tests_enabled")]
mod migration_tests {
    use log::debug;
    use starcoin_chain::ChainReader;
    use starcoin_config::{BuiltinNetworkID, ChainNetwork, DEFAULT_CACHE_SIZE};
    use starcoin_crypto::HashValue;
    use starcoin_data_migration::{
        do_migration, filter_chain_state_set, get_version_from_statedb, MigrationDataSet,
        MigrationExecutor,
    };
    use starcoin_state_api::{ChainStateReader, ChainStateWriter};
    use starcoin_statedb::ChainStateDB;
    use starcoin_storage::{storage::StorageInstance, Storage};
    use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
    use starcoin_types::account::Account;
    use starcoin_types::{
        account_address::AccountAddress, identifier::Identifier, state_set::ChainStateSet,
    };

    use starcoin_vm_types::{
        account_config::{association_address, genesis_address},
        on_chain_config::{access_path_for_config, Version},
        state_view::StateReaderExt,
        transaction::Transaction,
    };
    use std::sync::Arc;
    use tempfile::TempDir;
    use test_helper::{
        chain::gen_chain_for_test_and_return_statedb, create_block_with_transactions,
        print_bcs_decoded_resources, print_chain_state_set,
        txn::create_account_txn_sent_as_association,
    };

    fn test_vm1_net() -> anyhow::Result<ChainNetwork> {
        let chain_name = "vm1-testnet".to_string();
        ChainNetwork::new_custom(
            chain_name,
            124.into(),
            BuiltinNetworkID::Test.genesis_config().clone(),
            BuiltinNetworkID::Test.genesis_config2().clone(),
        )
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

    fn verify_migration_genesis_balance(
        statedb: &ChainStateDB,
        expect_balance: u128,
    ) -> anyhow::Result<()> {
        assert_eq!(
            statedb.get_balance(genesis_address())?.unwrap_or(0),
            expect_balance,
            "genesis balance should be {:?} after migration",
            expect_balance
        );
        Ok(())
    }

    /// Test function to demonstrate BCS resource printing
    #[stest::test]
    fn test_print_bcs_decoded_resources() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        // Use the test snapshot data
        let (file_name, data_hash, snapshot_pack) = MigrationDataSet::main_0x1().as_tuple();

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
        let temp_dir = starcoin_config::temp_dir();
        let (_block_chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &test_vm1_net()?,
                temp_dir.path().to_path_buf(),
            )?;

        assert_eq!(
            _block_chain
                .chain_state_reader()
                .get_balance(genesis_address())?
                .unwrap_or(0),
            10000
        );
        verify_migration_results(&statedb, 12)
    }

    /// Test basic migration functionality and file integrity
    #[stest::test]
    fn test_migration_basic_functionality_and_integrity() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let net = ChainNetwork::new_dev();
        let (_, chain_state_db) = gen_chain_for_test_and_return_statedb(&net, None)?;

        // Execute migration (this will verify file hash and basic functionality)
        let state_root = do_migration(&chain_state_db, net.chain_id())?;

        // Verify post-migration state
        let statedb = chain_state_db.fork_at(state_root);
        verify_migration_results(&statedb, 12)?;
        verify_migration_genesis_balance(&statedb, 10000)?;

        Ok(())
    }

    /// Test migration with comparing two chains
    #[stest::test]
    fn test_migration_with_comparing_two_chains() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let net = ChainNetwork::new_dev();
        let (_, chain_state_db) = gen_chain_for_test_and_return_statedb(&net, None)?;

        // First, execute migration to get the expected state
        let state_root1 = do_migration(&chain_state_db, net.chain_id())?;
        let statedb1 = chain_state_db.fork_at(state_root1);
        let exported_state_data = statedb1.dump()?;

        // Create a fresh test environment
        let (_net2, _storage2, chain_state_db) = create_test_environment(BuiltinNetworkID::Dev)?;
        chain_state_db.apply(exported_state_data)?;
        let state_root2 = chain_state_db.commit()?;
        chain_state_db.flush()?;
        let statedb2 = chain_state_db.fork_at(state_root2);

        // Since the exported state data already contains migrated data,
        // the state root should remain the same (idempotency)
        assert_eq!(
            state_root1, state_root2,
            "State root should compare eq with stateroot"
        );

        // Verify migration results
        verify_migration_results(&statedb1, 12)?;
        verify_migration_results(&statedb2, 12)?;

        verify_migration_genesis_balance(&statedb1, 10000)?;
        verify_migration_genesis_balance(&statedb2, 10000)?;

        Ok(())
    }

    /// Test migration idempotency
    #[stest::test]
    fn test_migration_idempotency() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let net = ChainNetwork::new_dev();
        let (_, chain_state_db) = gen_chain_for_test_and_return_statedb(&net, None)?;

        // Execute migration again
        let first_state_root = do_migration(&chain_state_db, net.chain_id())?;
        let first_statedb = chain_state_db.fork_at(first_state_root);
        let first_version = get_version_from_statedb(&first_statedb)?;
        let first_balance = first_statedb.get_balance(genesis_address())?.unwrap_or(0);

        // Second migration execution
        let second_state_root = do_migration(&first_statedb, net.chain_id())?;
        let second_statedb = first_statedb.fork_at(second_state_root);
        let second_version = get_version_from_statedb(&second_statedb)?;
        let second_balance = first_statedb.get_balance(genesis_address())?.unwrap_or(0);

        // Verify that both executions produce identical results
        assert_eq!(
            first_state_root, second_state_root,
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

        let dev_net = ChainNetwork::new_dev();
        let (_chain, statedb) = gen_chain_for_test_and_return_statedb(&dev_net, None)?;
        let state_root = do_migration(&statedb, dev_net.chain_id())?;
        let statedb_dev = statedb.fork_at(state_root);

        let test_net = ChainNetwork::new_test();
        let (_chain, statedb) = gen_chain_for_test_and_return_statedb(&test_net, None)?;
        let state_root = do_migration(&statedb, test_net.chain_id())?;
        let statedb_test = statedb.fork_at(state_root);

        verify_migration_results(&statedb_dev, 12)?;
        verify_migration_results(&statedb_test, 12)?;

        Ok(())
    }

    /// Test migration with 0x1 account state data specifically
    #[stest::test]
    fn test_migration_test_data() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let net = test_vm1_net()?;
        let temp_dir = starcoin_config::temp_dir();
        let (_block_chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &net,
                temp_dir.path().to_path_buf(),
            )?;

        // migration from genesis
        verify_migration_results(&statedb, 12)?;

        Ok(())
    }

    const TRANSAFER_AMOUNT: u128 = 1000000000;

    #[stest::test]
    fn test_dev_basic_peer_2_peer_after_migration_genesis() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        let net = test_vm1_net()?;
        let temp = TempDir::new()?;
        let (mut chain, statedb) =
            test_helper::chain::gen_chain_for_test_and_return_statedb_with_temp_storage(
                &net,
                temp.path().to_path_buf(),
            )?;

        debug!("test_dev_basic_peer_2_peer_after_migration_genesis | block_meta data: {:?}, timestamp: {:?}",
            statedb.get_block_metadata(),
            chain.time_service().now_secs(),
        );

        let random_account = Account::new();
        let p2p_txn = create_account_txn_sent_as_association(
            &random_account,
            0,
            TRANSAFER_AMOUNT,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            &net,
        );
        let (_executed_block, state_root) = create_block_with_transactions(
            &mut chain,
            &net,
            association_address(),
            vec![Transaction::UserTransaction(p2p_txn)],
        )?;

        let statedb = statedb.fork_at(state_root);
        assert_eq!(
            statedb.get_balance(*random_account.address())?.unwrap_or(0),
            TRANSAFER_AMOUNT
        );
        Ok(())
    }

    /// Test block mining simulation with data migration in memory
    /// This test simulates the process of building a blockchain in memory,
    /// mining an empty block, and verifying data migration results
    #[ignore]
    #[stest::test(timeout = 6000)]
    fn test_block_migration_with_blockchain_mining() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        // Create a test network (using Main to trigger migration)
        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Proxima);
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

        const MAX_TEST_BLOCKS: usize = 7;

        // Create N blocks (empty block)
        let mut latest_state_root = genesis_header.state_root();
        for i in 0..MAX_TEST_BLOCKS {
            debug!(
                "=== test_block_migration_with_blockchain_mining begin block {} ===",
                i
            );

            // Create block template for the first block (block #1) - empty block
            let (_executed_block, _stateroot) =
                create_block_with_transactions(&mut chain, &net, association_address(), vec![])?;

            debug!(
                "test_block_migration_with_blockchain_mining | executed_block header stateroot:{:?}, state_root1: {:?}, chain id: {:?}",
                _executed_block.header().state_root(),
                _executed_block.multi_state().state_root1(),
                statedb.get_chain_id()?,
            );
            latest_state_root = _stateroot;

            debug!(
                "=== test_block_migration_with_blockchain_mining end block {} ===",
                i
            );
        }

        let statedb = statedb.fork_at(latest_state_root);
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

    #[stest::test(timeout = 6000)]
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

        let net = ChainNetwork::new_builtin(BuiltinNetworkID::Dev);
        let (_chain, statedb) = gen_chain_for_test_and_return_statedb(&net, None)?;

        let address = AccountAddress::ONE;

        let account_state = statedb
            .get_account_state_set(&address)?
            .expect("get account state set should successfully");
        let chain_state_set = ChainStateSet::new(vec![(address, account_state)]);
        let filtered_chain_state_set = filter_chain_state_set(chain_state_set, &statedb)?;
        statedb.apply(filtered_chain_state_set)?;

        Ok(())
    }

    #[stest::test]
    pub fn test_print_account_state_set() -> anyhow::Result<()> {
        let data_set = MigrationExecutor::read_chain_stateset_from_migration_dataset(
            MigrationDataSet::main(),
        )?;

        let account_address =
            AccountAddress::from_hex_literal("0x8c109349c6bd91411d6bc962e080c4a3")?;
        print_chain_state_set(&data_set, Some(account_address))?;

        Ok(())
    }
}
