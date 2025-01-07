/// Asset Mapping Module
/// This module implements functionality for managing fungible asset mappings in the Starcoin framework.
/// It provides capabilities for creating stores, managing balances, and assigning assets to accounts
/// with proof verification.
module starcoin_framework::asset_mapping {

    use std::error;
    use std::signer;
    use std::string;
    use std::vector;

    use starcoin_framework::coin;
    use starcoin_framework::fungible_asset::{Self, FungibleStore, Metadata};
    use starcoin_framework::object::{Self, ExtendRef, Object};
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_proof_verifier;
    use starcoin_framework::stc_util;
    use starcoin_framework::system_addresses;
    use starcoin_std::debug;
    use starcoin_std::smart_table;

    #[test_only]
    use starcoin_framework::account;
    #[test_only]
    use starcoin_framework::starcoin_coin::{Self, STC};
    #[test_only]
    use starcoin_framework::starcoin_proof_verifier::splite_symbol;
    #[test_only]
    use starcoin_std::type_info;

    /// AssetMappingStore represents a store for mapped assets
    /// Contains:
    /// - extend_ref: Reference for extending object capabilities
    /// - fungible_store: The actual store holding fungible assets
    struct AssetMappingStore has key, store {
        extend_ref: ExtendRef,
        fungible_store: Object<FungibleStore>,
    }

    /// AssetMappingPool manages a collection of asset mapping stores
    /// Contains:
    /// - proof_root: Root hash for proof verification
    /// - anchor_height: Block height anchor for the mapping
    /// - token_stores: Smart table mapping metadata to stores
    struct AssetMappingPool has key, store {
        token_stores: smart_table::SmartTable<Object<Metadata>, AssetMappingStore>,
    }

    struct AssetMappingProof has key, store {
        proof_root: vector<u8>,
    }

    /// AssetMappingCoinType represents a mapping that from old version token types to now version asset stores
    /// eg. 0x1::STC::STC -> 0x1::starcoin_coin::STC
    ///
    struct AssetMappingCoinType has key, store {
        token_mapping: smart_table::SmartTable<std::string::String, Object<Metadata>>,
    }

    /// Error code for invalid signer
    const EINVALID_SIGNER: u64 = 101;
    const EINVALID_NOT_PROOF: u64 = 102;
    const EINVALID_PROOF_ROOT: u64 = 102;

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
            token_stores: smart_table::new<Object<Metadata>, AssetMappingStore>(),
        });
        move_to(framework, AssetMappingProof {
            proof_root,
        });
        move_to(framework, AssetMappingCoinType {
            token_mapping: smart_table::new<string::String, Object<Metadata>>(),
        })
    }

    /// Creates a new store from a coin
    /// @param token_issuer - The token issuer signer
    /// @param coin - The coin to be stored
    /// Requirements:
    /// - Token issuer must be authorized for the given token type
    /// - Converts coin to fungible asset and stores it
    public fun create_store_from_coin<T: key>(
        token_issuer: &signer,
        old_token_str: string::String,
        coin: coin::Coin<T>
    ) acquires AssetMappingPool, AssetMappingCoinType {
        let token_issuer_addr = signer::address_of(token_issuer);
        assert!(
            token_issuer_addr == stc_util::token_issuer<T>(),
            error::unauthenticated(EINVALID_SIGNER)
        );

        let fungible_asset = coin::coin_to_fungible_asset(coin);
        let token_stores =
            &mut borrow_global_mut<AssetMappingPool>(system_addresses::get_starcoin_framework()).token_stores;

        let (metadata, fungible_store, extend_ref) = create_store_for_type<T>(token_issuer);
        fungible_asset::deposit(fungible_store, fungible_asset);
        smart_table::add(token_stores, metadata, AssetMappingStore {
            extend_ref,
            fungible_store,
        });

        // Add token mapping coin type
        let asset_coin_type =
            borrow_global_mut<AssetMappingCoinType>(system_addresses::get_starcoin_framework());
        smart_table::add(&mut asset_coin_type.token_mapping, old_token_str, metadata);
    }

    /// Creates a store for a specific token type
    /// @param framework - The framework signer
    /// @returns (metadata, store, extend_ref):
    /// - metadata: Token metadata object
    /// - store: Created fungible store
    /// - extend_ref: Extension reference for the store
    fun create_store_for_type<T>(framework: &signer): (Object<Metadata>, Object<FungibleStore>, ExtendRef) {
        debug::print(&std::string::utf8(b"asset_mapping::create_store_for_type | entered"));

        let metadata = coin::ensure_paired_metadata<T>();
        let construct_ref = object::create_object_from_account(framework);

        let store = fungible_asset::create_store(&construct_ref, metadata);

        // Generate extend reference
        let extend_ref = object::generate_extend_ref(&construct_ref);
        debug::print(&std::string::utf8(b"asset_mapping::create_store_for_type | exited"));

        (metadata, store, extend_ref)
    }

    /// Retrieves the balance for a specific token type
    /// @returns Current balance of the token in the mapping pool
    fun balance<T>(): u64 acquires AssetMappingPool {
        let metadata = coin::ensure_paired_metadata<T>();
        let pool = borrow_global<AssetMappingPool>(system_addresses::get_starcoin_framework());
        fungible_asset::balance(smart_table::borrow(&pool.token_stores, metadata).fungible_store)
    }

    public entry fun assign_to_account_with_proof(
        token_issuer: &signer,
        receiper: address,
        old_token_str: vector<u8>,
        proof_path_hash: vector<u8>,
        proof_value_hash: vector<u8>,
        proof_siblings: vector<u8>,
        amount: u64
    ) acquires AssetMappingPool, AssetMappingCoinType, AssetMappingProof {
        assert!(
            exists<AssetMappingProof>(system_addresses::get_starcoin_framework()),
            error::invalid_state(EINVALID_PROOF_ROOT)
        );

        // Verify that the token type of the request mapping is the passed-in verification type
        assert!(
            calculation_proof(proof_path_hash, proof_value_hash, split_proof_siblings_from_vec(proof_siblings)),
            error::unauthenticated(EINVALID_NOT_PROOF)
        );

        assign_to_account(token_issuer, receiper, old_token_str, amount);
    }

    /// Assigns tokens to a recipient account with proof verification
    /// @param token_issuer - The token issuer signer
    /// @param receiper - Recipient address
    /// @param proove - Proof data for verification
    /// @param amount - Amount of tokens to assign
    /// Requirements:
    /// - Valid proof must be provided
    /// - Sufficient balance must exist
    fun assign_to_account(
        token_issuer: &signer,
        receiper: address,
        old_token_str: vector<u8>,
        amount: u64
    ) acquires AssetMappingPool, AssetMappingCoinType {
        let coin_type_mapping = borrow_global<AssetMappingCoinType>(system_addresses::get_starcoin_framework());
        let metadata = smart_table::borrow(&coin_type_mapping.token_mapping, string::utf8(old_token_str));
        let mapping_pool = borrow_global_mut<AssetMappingPool>(signer::address_of(token_issuer));
        let mapping_store = smart_table::borrow_mut(
            &mut mapping_pool.token_stores,
            *metadata
        );

        let store_signer = object::generate_signer_for_extending(&mapping_store.extend_ref);
        fungible_asset::deposit(
            primary_fungible_store::ensure_primary_store_exists(receiper, *metadata),
            fungible_asset::withdraw(&store_signer, mapping_store.fungible_store, amount)
        )
    }

    fun split_proof_siblings_from_vec(_siblings: vector<u8>): vector<vector<u8>> {
        // TODO(BobOng): implement this function
        vector::empty()
    }

    /// Computes and verifies the provided proof
    fun calculation_proof(
        proof_path_hash: vector<u8>,
        blob_hash: vector<u8>,
        proof_siblings: vector<vector<u8>>
    ): bool acquires AssetMappingProof {
        let expect_proof_root =
            borrow_global_mut<AssetMappingProof>(system_addresses::get_starcoin_framework()).proof_root;
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
    ) acquires AssetMappingPool, AssetMappingCoinType {
        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | entered"));

        let amount = 10000000000;
        Self::initialize(framework, vector::empty<u8>());

        // create genesis account
        account::create_account_for_test(signer::address_of(framework));

        let (burn_cap, mint_cap) = starcoin_coin::initialize_for_test(framework);
        coin::register<STC>(framework);
        starcoin_coin::mint(framework, signer::address_of(framework), amount);


        // Construct Old token string
        let old_token_str = string::utf8(b"0x00000000000000000000000000000001::starcoin_coin::STC");

        let coin = coin::withdraw<STC>(framework, amount);
        create_store_from_coin<STC>(
            framework,
            old_token_str,
            coin
        );
        assert!(Self::balance<STC>() == amount, 10001);

        // Assign to alice
        let alice_addr = signer::address_of(alice);
        assign_to_account(
            framework,
            alice_addr,
            *string::bytes(&old_token_str),
            amount
        );
        assert!(Self::balance<STC>() == 0, 10002);

        let stc_metadata = coin::ensure_paired_metadata<STC>();
        assert!(primary_fungible_store::balance(alice_addr, stc_metadata) == amount, 10003);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);

        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | exited"));
    }

    #[test(framework= @starcoin_framework)]
    fun test_asset_mapping_calculation_proof(framework: &signer) acquires AssetMappingProof {
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
}
