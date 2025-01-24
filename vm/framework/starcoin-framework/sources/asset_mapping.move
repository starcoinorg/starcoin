/// Asset Mapping Module
/// This module implements functionality for managing fungible asset mappings in the Starcoin framework.
/// It provides capabilities for creating stores, managing balances, and assigning assets to accounts
/// with proof verification.
module starcoin_framework::asset_mapping {

    use std::error;
    use std::signer;
    use std::string;

    use starcoin_framework::coin;
    use starcoin_framework::fungible_asset::{Self, FungibleStore, Metadata};
    use starcoin_framework::object::{Self, ExtendRef, Object};
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_proof_verifier;
    use starcoin_framework::stc_util;
    use starcoin_framework::system_addresses;
    use starcoin_std::debug;
    use starcoin_std::simple_map::{Self, SimpleMap};
    #[test_only]
    use std::hash;

    #[test_only]
    use std::vector;
    #[test_only]
    use starcoin_framework::account;
    #[test_only]
    use starcoin_framework::starcoin_coin::{Self, STC};
    #[test_only]
    use starcoin_framework::starcoin_proof_verifier::splite_symbol;
    #[test_only]
    use starcoin_std::type_info;

    #[resource_group_member(group = starcoin_framework::object::ObjectGroup)]
    /// AssetMappingStore represents a store for mapped assets
    /// Contains:
    /// - extend_ref: Reference for extending object capabilities
    /// - fungible_store: The actual store holding fungible assets
    /// - fungible_metadata: The type of fungible assets
    struct AssetMappingStore has key, store {
        extend_ref: ExtendRef,
        fungible_store: Object<FungibleStore>,
        metadata: Object<Metadata>
    }

    struct AssetMappingStoreT<phantom T> has key {
        coin: coin::Coin<T>,
        old_path_str: vector<u8>,
    }

    /// AssetMappingCoinType represents a mapping that from old version token types to now version asset stores
    /// eg. 0x1::STC::STC -> 0x1::starcoin_coin::STC
    ///
    struct AssetMappingPool has key, store {
        proof_root: vector<u8>,
        token_mapping: SimpleMap<std::string::String, address>,
    }

    /// Error code for invalid signer
    const EINVALID_SIGNER: u64 = 101;
    const EINVALID_PROOF_ROOT: u64 = 102;
    const EINVALID_NOT_PROOF: u64 = 103;
    const EINVALID_ASSET_MAPPING_POOL: u64 = 104;
    const EINVALID_DEPOSIT: u64 = 104;

    const ASSET_MAPPING_OBJECT_SEED: vector<u8> = b"asset-mapping";

    /// Initializes the asset mapping pool
    /// @param framework - The framework signer
    /// @param proof_root - Initial proof root for verification
    /// Verifies the framework signer and creates a new AssetMappingPool
    public fun initialize(framework: &signer, proof_root: vector<u8>) {
        assert!(
            signer::address_of(framework) == system_addresses::get_starcoin_framework(),
            error::unauthenticated(EINVALID_SIGNER)
        );
        move_to(framework, AssetMappingPool {
            token_mapping: simple_map::new(),
            proof_root,
        });
    }


    /// Creates a new store from a coin
    /// @param token_issuer - The token issuer signer
    /// @param coin - The coin to be stored
    /// Requirements:
    /// - Token issuer must be authorized for the given token type
    /// - Converts coin to fungible asset and stores it
    public fun create_store_from_coin<T: key>(
        token_issuer: &signer,
        old_token_str: vector<u8>,
        coin: coin::Coin<T>
    ) acquires AssetMappingPool {
        debug::print(&string::utf8(b"asset_mapping::create_store_from_coin | entered"));

        let token_issuer_addr = signer::address_of(token_issuer);
        assert!(
            token_issuer_addr == stc_util::token_issuer<T>(),
            error::unauthenticated(EINVALID_SIGNER)
        );

        debug::print(&string::utf8(b"asset_mapping::create_store_from_coin | coin_to_fungible_asset"));

        let fungible_asset = coin::coin_to_fungible_asset(coin);

        let (
            metadata,
            fungible_store,
            extend_ref
        ) = create_store_for_coin_type<T>(token_issuer);

        debug::print(&string::utf8(b"asset_mapping::create_store_from_coin | created token store"));
        debug::print(&fungible_store);

        fungible_asset::deposit(fungible_store, fungible_asset);

        // Add token mapping coin type
        let asset_coin_type =
            borrow_global_mut<AssetMappingPool>(system_addresses::get_starcoin_framework());

        let store_constructor_ref = &object::create_object(system_addresses::get_core_resource_address());
        let store_signer = &object::generate_signer(store_constructor_ref);
        move_to(store_signer, AssetMappingStore {
            extend_ref,
            fungible_store,
            metadata,
        });

        simple_map::add(
            &mut asset_coin_type.token_mapping,
            string::utf8(old_token_str),
            object::address_from_constructor_ref(store_constructor_ref),
        );

        debug::print(&string::utf8(b"asset_mapping::create_store_from_coin | exited"));
    }

    /// Creates a store for a specific token type
    /// @param framework - The framework signer
    /// @returns (metadata, store, extend_ref):
    /// - metadata: Token metadata object
    /// - store: Created fungible store
    /// - extend_ref: Extension reference for the store
    fun create_store_for_coin_type<T>(account: &signer): (Object<Metadata>, Object<FungibleStore>, ExtendRef) {
        debug::print(&std::string::utf8(b"asset_mapping::create_store_for_type | entered"));

        let metadata = coin::ensure_paired_metadata<T>();
        let construct_ref = object::create_object_from_account(account);

        let store = fungible_asset::create_store(&construct_ref, metadata);

        // Generate extend reference
        let extend_ref = object::generate_extend_ref(&construct_ref);
        debug::print(&std::string::utf8(b"asset_mapping::create_store_for_type | exited"));

        (metadata, store, extend_ref)
    }

    /// Retrieves the balance for a specific token type
    /// @returns Current balance of the token in the mapping pool
    fun fungible_store_balance(old_asset_str: vector<u8>): u64 acquires AssetMappingPool, AssetMappingStore {
        let pool = borrow_global<AssetMappingPool>(system_addresses::get_starcoin_framework());
        let store_object_addr = simple_map::borrow(&pool.token_mapping, &string::utf8(old_asset_str));
        let mapping_store = borrow_global<AssetMappingStore>(*store_object_addr);
        fungible_asset::balance(mapping_store.fungible_store)
    }

    public entry fun assign_to_account_with_proof(
        token_issuer: &signer,
        receiper: address,
        old_token_str: vector<u8>,
        proof_path_hash: vector<u8>,
        proof_value_hash: vector<u8>,
        proof_siblings: vector<u8>,
        amount: u64
    ) acquires AssetMappingPool, AssetMappingStore {
        assert!(
            exists<AssetMappingPool>(system_addresses::get_starcoin_framework()),
            error::invalid_state(EINVALID_PROOF_ROOT)
        );

        // Verify that the token type of the request mapping is the passed-in verification type
        assert!(
            calculation_proof(proof_path_hash, proof_value_hash, starcoin_proof_verifier::split(proof_siblings)),
            error::unauthenticated(EINVALID_NOT_PROOF)
        );

        assign_to_account(token_issuer, receiper, old_token_str, amount);
    }

    public entry fun assign_to_account_test(
        system_account: &signer,
        receiver: address,
        old_token_str: vector<u8>,
        amount: u64
    ) acquires AssetMappingPool, AssetMappingStore {
        Self::assign_to_account(system_account, receiver, old_token_str, amount);
    }

    /// Assigns tokens to a recipient account with proof verification
    /// @param token_issuer - The token issuer signer
    /// @param receiper - Recipient address
    /// @param proove - Proof data for verification
    /// @param amount - Amount of tokens to assign
    /// Requirements:
    /// - Valid proof must be provided
    /// - Sufficient balance must exist
    public fun assign_to_account(
        system_account: &signer,
        receiver: address,
        old_token_str: vector<u8>,
        amount: u64
    ) acquires AssetMappingPool, AssetMappingStore {
        debug::print(&string::utf8(b"asset_mapping::assign_to_account | entered"));

        let account_addr = signer::address_of(system_account);
        assert!(
            system_addresses::is_starcoin_framework_address(account_addr) ||
                system_addresses::is_core_resource_address(account_addr),
            EINVALID_SIGNER
        );

        assert!(
            exists<AssetMappingPool>(system_addresses::get_starcoin_framework()),
            error::invalid_state(EINVALID_ASSET_MAPPING_POOL)
        );

        let coin_type_mapping =
            borrow_global_mut<AssetMappingPool>(system_addresses::get_starcoin_framework());
        debug::print(&string::utf8(b"asset_mapping::assign_to_account | coin_type_mapping"));
        debug::print(&coin_type_mapping.token_mapping);

        let mapping_store_addr = simple_map::borrow(&coin_type_mapping.token_mapping, &string::utf8(old_token_str));
        debug::print(mapping_store_addr);
        let mapping_store = borrow_global<AssetMappingStore>(*mapping_store_addr);

        // debug::print(&string::utf8(b"asset_mapping::assign_to_account | metadata"));
        // debug::print(&fungible_asset::is_frozen(mapping_store.fungible_store));

        debug::print(&string::utf8(b"asset_mapping::assign_to_account | fungible_asset::withdraw"));
        let mapping_fa = fungible_asset::withdraw(
            &object::generate_signer_for_extending(&mapping_store.extend_ref),
            mapping_store.fungible_store,
            amount
        );
        debug::print(&string::utf8(b"asset_mapping::assign_to_account | Getting receiver fungible store: "));
        debug::print(&mapping_fa);

        let mapping_fa_amount = fungible_asset::amount(&mapping_fa);

        let target_store =
            primary_fungible_store::ensure_primary_store_exists(receiver, mapping_store.metadata);
        fungible_asset::deposit(target_store, mapping_fa);

        let target_store_balance = fungible_asset::balance(target_store);
        debug::print(&string::utf8(b"asset_mapping::assign_to_account | target_store balance: "));
        debug::print(&target_store);
        debug::print(&target_store_balance);

        assert!(
            target_store_balance >= mapping_fa_amount,
            error::invalid_state(EINVALID_DEPOSIT)
        );
        debug::print(&string::utf8(b"asset_mapping::assign_to_account | exited"));
    }

    /// Computes and verifies the provided proof
    fun calculation_proof(
        proof_path_hash: vector<u8>,
        blob_hash: vector<u8>,
        proof_siblings: vector<vector<u8>>
    ): bool acquires AssetMappingPool {
        let expect_proof_root =
            borrow_global_mut<AssetMappingPool>(system_addresses::get_starcoin_framework()).proof_root;
        let actual_root = starcoin_proof_verifier::computer_root_hash(
            proof_path_hash,
            blob_hash,
            proof_siblings
        );
        expect_proof_root == actual_root
    }


    // Test function for asset mapping store creation and assignment
    // Tests
    //  Store creation from coin
    //  Balance checking
    //  Asset assignment to account
    //  Final balance verification
    #[test(framework= @starcoin_framework, alice= @0x123)]
    fun test_asset_mapping_create_store_from_coin(
        framework: &signer,
        alice: &signer
    ) acquires AssetMappingPool, AssetMappingStore {
        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | entered"));

        let amount = 10000000000;
        Self::initialize(framework, vector::empty<u8>());

        debug::print(
            &std::string::utf8(
                b"asset_mapping::test_asset_mapping_create_store_from_coin | before create_account_for_test"
            )
        );
        // create genesis account
        account::create_account_for_test(signer::address_of(framework));

        debug::print(
            &std::string::utf8(
                b"asset_mapping::test_asset_mapping_create_store_from_coin | starcoin_coin::initialize_for_test"
            )
        );

        let (burn_cap, mint_cap) = starcoin_coin::initialize_for_test(framework);

        debug::print(
            &std::string::utf8(
                b"asset_mapping::test_asset_mapping_create_store_from_coin | coin::register<STC>(framework)"
            )
        );
        coin::register<STC>(framework);

        debug::print(
            &std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | starcoin_coin::mint")
        );
        starcoin_coin::mint(framework, signer::address_of(framework), amount);

        debug::print(
            &std::string::utf8(
                b"asset_mapping::test_asset_mapping_create_store_from_coin | after coin::register<STC>(framework) and mint"
            )
        );

        // Construct Old token string
        let old_token_str = b"0x00000000000000000000000000000001::starcoin_coin::STC";
        let coin = coin::withdraw<STC>(framework, amount);
        Self::create_store_from_coin<STC>(
            framework,
            old_token_str,
            coin
        );
        assert!(Self::fungible_store_balance(old_token_str) == amount, 10001);

        // Assign to alice
        let alice_addr = signer::address_of(alice);
        Self::assign_to_account(
            framework,
            alice_addr,
            old_token_str,
            amount
        );
        assert!(Self::fungible_store_balance(old_token_str) == 0, 10002);

        let stc_metadata = coin::ensure_paired_metadata<STC>();
        assert!(primary_fungible_store::balance(alice_addr, stc_metadata) == amount, 10003);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);

        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | exited"));
    }

    #[test(framework= @starcoin_framework)]
    fun test_asset_mapping_calculation_proof(framework: &signer) acquires AssetMappingPool {
        let siblings_data = vector::empty<u8>();
        vector::append(&mut siblings_data, x"cfb1462d4fc72f736eab2a56b2bf72ca6ad1c4e8c79557046a8b0adce047f007");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"5ca9febe74c7fde3fdcf2bd464de6d8899a0a13d464893aada2714c6fa774f9d");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"1519a398fed69687cabf51adf831f0ee1650aaf79775d00135fc70f55a73e151");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"50ce5c38983ba2eb196acd44e0aaedf040b1437ad1106e05ca452d7e27e4e03f");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"55ed28435637a061a6dd9e20b72849199cd36184570f976b7e306a27bebf2fdf");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"0dc23e31614798a6f67659b0b808b3eadc3b13a2a7bc03580a9e3004e45c2e6c");
        vector::push_back(&mut siblings_data, splite_symbol());

        vector::append(&mut siblings_data, x"83bed048bc0bc452c98cb0e9f1cc0f691919eaf756864fc44940c2d1e01da92a");
        vector::push_back(&mut siblings_data, splite_symbol());

        let siblings = starcoin_proof_verifier::split(siblings_data);

        let element_key = x"4cc8bd9df94b37c233555d9a3bba0a712c3c709f047486d1e624b2bcd3b83266";
        Self::initialize(framework, x"f65860f575bf2a198c069adb4e7872037e3a329b63ef617e40afa39b87b067c8");
        assert!(
            Self::calculation_proof(
                element_key,
                x"4f2b59b9af93b435e0a33b6ab7a8a90e471dba936be2bc2937629b7782b8ebd0",
                siblings
            ),
            10010
        );
    }

    #[test]
    fun test_asset_mapping_proof_coin_type_name() {
        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_coin_type_verify | entered"));

        // Check type path name
        let type_name = type_info::type_name<coin::CoinStore<starcoin_coin::STC>>();
        debug::print(
            &std::string::utf8(
                b"asset_mapping::test_asset_mapping_coin_type_verify | type of coin::CoinStore<starcoin_coin::STC>"
            )
        );
        debug::print(&type_name);
        assert!(
            type_name == std::string::utf8(
                b"0x00000000000000000000000000000001::coin::CoinStore<0x00000000000000000000000000000001::starcoin_coin::STC>"
            ),
            10020
        );
        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_coin_type_verify | exited"));
    }

    #[test]
    fun test_calculation_proof_1() {
        let proof_sibling_data = vector::empty<vector<u8>>();
        vector::push_back(&mut proof_sibling_data, x"6b67362f680d4d15f996aed2a5c83e3dce37cb37bed4bc498aaeef77ea8a28a2");
        vector::push_back(&mut proof_sibling_data, x"5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000");
        vector::push_back(&mut proof_sibling_data, x"5ca9febe74c7fde3fdcf2bd464de6d8899a0a13d464893aada2714c6fa774f9d");
        vector::push_back(&mut proof_sibling_data, x"06fa88f7fae77461044d10cc504c5e6666910d1ee4d1b1d99f8dbea047d0c9ff");
        vector::push_back(&mut proof_sibling_data, x"5f3620db0071243d18285e1a2c4d74b734421e65581bbb41e70498369c863cdb");
        vector::push_back(&mut proof_sibling_data, x"4949e6d0a2be6d8a79fd3fee859e10e564815e88a16dec26760be15c8ae017e7");
        vector::push_back(&mut proof_sibling_data, x"8cd8632ea21b3a4623bb825d2451f6c76055cda7433e1da3d76773dba7c06878");
        vector::push_back(&mut proof_sibling_data, x"379f1d32988ebd8d01627d0326523e28aa5fa1dbf2e87d076d7dca72884a4c46");

        assert!(starcoin_proof_verifier::computer_root_hash(
            x"9afe1e0e6013eb63b6004a4eb6b1bf76bdb04b725619648163d9dbc3194f224c",
            x"b9d3ba6fe71eff0b1e9c9d70401fd6767a15a82a28f2542dbc27fda50730b6e9",
            proof_sibling_data,
        ) == x"a307d98b0b6da330fb0ac31283d6913d18627412a515b0c88e59346dfe04e0d5", 10011);
    }

    #[test]
    fun test_asset_mapping_hello_hash() {
        debug::print(&hash::sha3_256(b"hello"));
    }
}
