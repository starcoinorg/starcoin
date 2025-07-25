// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod export_test {
    use resource_code_exporter::export::export_from_statedb;
    use starcoin_chain::ChainReader;
    use starcoin_logger::prelude::info;
    use starcoin_types::account::Account;
    use starcoin_types::transaction::{Transaction, TransactionPayload};
    use starcoin_types::{language_storage::StructTag, state_set::ChainStateSet};
    use starcoin_vm_types::account_config::{
        association_address, genesis_address, token_code::TokenCode, BalanceResource,
        STC_TOKEN_CODE_STR,
    };
    use starcoin_vm_types::state_view::StateReaderExt;
    use starcoin_vm_types::transaction::Package;
    use std::path::Path;
    use std::str::FromStr;
    use test_helper::{
        block::create_block_with_transactions,
        chain::{gen_chain_for_test_and_return_statedb, vm1_testnet},
        executor::{compile_modules_with_address, prepare_genesis},
        txn::create_account_txn_sent_as_association,
    };

    #[stest::test]
    fn test_export_from_statedb() -> anyhow::Result<()> {
        // Initialize logger for test
        starcoin_logger::init_for_test();

        // Initialize test storage with genesis
        let (chain_statedb, _net) = prepare_genesis();

        // Test BCS export
        let test_bcs_path = Path::new("test_dump_state.bcs");
        export_from_statedb(&chain_statedb, test_bcs_path, None)?;

        // Verify the BCS file was created and contains data
        assert!(test_bcs_path.exists(), "BCS file should be created");
        let file_size = std::fs::metadata(test_bcs_path)?.len();
        assert!(file_size > 0, "BCS file should not be empty");

        // Read back the BCS file and verify data integrity
        info!("Reading back BCS file for verification...");
        let bcs_data = std::fs::read(test_bcs_path)?;
        let deserialized_state: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;

        // Verify that the deserialized state contains data
        assert!(
            !deserialized_state.is_empty(),
            "Deserialized state should not be empty"
        );
        info!(
            "Successfully deserialized {} account states",
            deserialized_state.len()
        );

        // Check if association account exists and has balance
        let association_addr = association_address();
        let mut found_association = false;
        let mut association_balance = None;

        for (address, account_state_set) in deserialized_state.state_sets() {
            if *address != association_addr {
                continue;
            }

            found_association = true;
            info!("Found association account in exported state");

            let stc_balance_resource = BalanceResource::struct_tag_for_token(
                TokenCode::from_str(STC_TOKEN_CODE_STR)?.try_into()?,
            );

            // Check if association account has resource data
            if let Some(resource_set) = account_state_set.resource_set() {
                info!("Association account has {} resources", resource_set.len());

                // Look for balance resource in the resource set
                for (key, value) in resource_set.iter() {
                    // The balance resource key typically contains "Balance" in the path
                    let struct_tag: StructTag = bcs_ext::from_bytes::<StructTag>(key)?;

                    if struct_tag == stc_balance_resource {
                        info!("Found balance resource for association account");
                        // Try to deserialize as BalanceResource
                        match bcs_ext::from_bytes::<BalanceResource>(value) {
                            Ok(balance_resource) => {
                                association_balance = Some(balance_resource.token());
                                info!("Association account balance: {}", balance_resource.token());
                                break;
                            }
                            Err(e) => {
                                info!("Failed to deserialize balance resource: {}", e);
                            }
                        }
                    }
                }
            }
            break;
        }

        assert!(
            found_association,
            "Association account should exist in exported state"
        );
        if let Some(balance) = association_balance {
            assert!(
                balance > 0,
                "Association account should have positive balance, got: {}",
                balance
            );
        } else {
            info!("Could not verify association account balance, but account exists");
        }

        // Check 0x1 code count has 85
        let genesis_addr = genesis_address();
        let mut found_genesis = false;
        let mut genesis_code_count = 0;

        for (account, state_set) in deserialized_state.state_sets() {
            if *account == genesis_addr {
                found_genesis = true;
                info!("Found genesis account (0x1) in exported state");

                if let Some(code_set) = state_set.code_set() {
                    genesis_code_count = code_set.len();
                    info!("Genesis account has {} code modules", genesis_code_count);
                } else {
                    info!("Genesis account has no code_set");
                }
                break;
            }
        }

        assert!(
            found_genesis,
            "Genesis account (0x1) should exist in exported state"
        );
        assert_eq!(
            genesis_code_count, 85,
            "Genesis account (0x1) should have exactly 85 code modules, but found {}",
            genesis_code_count
        );

        // Clean up test file
        std::fs::remove_file(test_bcs_path)?;

        Ok(())
    }

    #[stest::test]
    fn test_export_account_code_state() -> anyhow::Result<()> {
        // Initialize logger for test
        starcoin_logger::init_for_test();

        // 1. Build blockchain
        let net = vm1_testnet()?;
        let (mut chain, statedb) = gen_chain_for_test_and_return_statedb(&net, None)?;

        info!("Blockchain created successfully");

        // 2. Create random account
        let random_account = Account::new();
        let random_account_address = *random_account.address();

        info!("Created random account: {}", random_account_address);

        // Create account using create_block_with_transactions
        let miner_account = association_address();
        let expire_time = net.time_service().now_secs() + 60 * 60;

        info!("=== Block1: create random account ===");
        create_block_with_transactions(
            &mut chain,
            &net,
            miner_account,
            vec![Transaction::UserTransaction(
                create_account_txn_sent_as_association(
                    &random_account,
                    0,          // sequence number
                    50_000_000, // initial amount
                    expire_time,
                    &net,
                ),
            )],
        )?;

        // Verify account was created
        assert!(chain
            .chain_state_reader()
            .get_account_resource(random_account_address)?
            .is_some());

        info!("Random account created successfully");

        // 3. Deploy a contract to the random account
        let module_source = r#"
        module {{sender}}::TestModule {
            struct TestResource has key, store {
                value: u64,
            }

            public fun init(account: &signer) {
                let resource = TestResource { value: 42 };
                move_to(account, resource);
            }

            public fun get_value(): u64 acquires TestResource {
                let resource = borrow_global<TestResource>(@{{sender}});
                resource.value
            }
        }
    "#;

        // Compile module with random account's address
        let compiled_modules = compile_modules_with_address(random_account_address, module_source);
        assert!(
            !compiled_modules.is_empty(),
            "Should compile at least one module"
        );

        let module = compiled_modules.into_iter().next().unwrap();

        info!(
            "Module compiled successfully for address: {}",
            random_account_address
        );

        // Deploy contract using create_block_with_transactions
        let package = Package::new_with_module(module)?;

        info!("=== Block2: deploy contract ===");
        create_block_with_transactions(
            &mut chain,
            &net,
            miner_account,
            vec![Transaction::UserTransaction(
                random_account.create_signed_txn_impl(
                    random_account_address,
                    TransactionPayload::Package(package),
                    0,       // sequence number for random account
                    100_000, // max gas amount
                    1,       // gas unit price
                    expire_time,
                    net.chain_id(),
                ),
            )],
        )?;

        info!("Contract deployed successfully to random account");

        // 4. Export data
        let test_bcs_path = Path::new("test_export_account_code_state.bcs");
        // Create a new statedb at the latest state_root
        let latest_state_root = chain.chain_state_reader().state_root();
        let export_statedb = statedb.fork_at(latest_state_root);
        export_from_statedb(&export_statedb, test_bcs_path, None)?;

        // Verify the BCS file was created and contains data
        assert!(test_bcs_path.exists(), "BCS file should be created");
        let file_size = std::fs::metadata(test_bcs_path)?.len();
        assert!(file_size > 0, "BCS file should not be empty");

        // Read back the BCS file and verify data integrity
        info!("Reading back BCS file for verification...");
        let bcs_data = std::fs::read(test_bcs_path)?;
        let deserialized_state: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;

        // Verify that the deserialized state contains data
        assert!(
            !deserialized_state.is_empty(),
            "Deserialized state should not be empty"
        );
        info!(
            "Successfully deserialized {} account states",
            deserialized_state.len()
        );

        // Check if random account exists and has code
        let mut found_random_account = false;
        let mut random_account_code_count = 0;

        for (account, state_set) in deserialized_state.state_sets() {
            if *account == random_account_address {
                found_random_account = true;
                info!("Found random account in exported state: {}", account);

                if let Some(code_set) = state_set.code_set() {
                    random_account_code_count = code_set.len();
                    info!(
                        "Random account has {} code modules",
                        random_account_code_count
                    );

                    // Print details about the code modules
                    for (i, (key, value)) in code_set.iter().enumerate() {
                        info!(
                            "Code module {}: key length = {}, value length = {}",
                            i,
                            key.len(),
                            value.len()
                        );
                    }
                } else {
                    info!("Random account has no code_set");
                }
                break;
            }
        }

        assert!(
            found_random_account,
            "Random account should exist in exported state"
        );
        assert!(
            random_account_code_count > 0,
            "Random account should have at least 1 code module, but found {}",
            random_account_code_count
        );

        info!(
            "Random account code verification passed! Found {} code modules",
            random_account_code_count
        );

        // Also verify that genesis account still has 85 modules
        let genesis_addr = genesis_address();
        let mut found_genesis = false;
        let mut genesis_code_count = 0;

        for (account, state_set) in deserialized_state.state_sets() {
            if *account == genesis_addr {
                found_genesis = true;
                info!("Found genesis account (0x1) in exported state");

                if let Some(code_set) = state_set.code_set() {
                    genesis_code_count = code_set.len();
                    info!("Genesis account has {} code modules", genesis_code_count);
                } else {
                    info!("Genesis account has no code_set");
                }
                break;
            }
        }

        assert!(
            found_genesis,
            "Genesis account (0x1) should exist in exported state"
        );
        assert_eq!(
            genesis_code_count, 85,
            "Genesis account (0x1) should have exactly 85 code modules, but found {}",
            genesis_code_count
        );

        // Clean up test file
        std::fs::remove_file(test_bcs_path)?;

        info!("test_export_account_code_state completed successfully");
        Ok(())
    }
}
