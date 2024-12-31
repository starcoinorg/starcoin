/// Asset Mapping Module
/// This module implements functionality for managing fungible asset mappings in the Starcoin framework.
/// It provides capabilities for creating stores, managing balances, and assigning assets to accounts
/// with proof verification.
module starcoin_framework::asset_mapping {

    use std::error;
    use std::signer;
    use std::vector;

    use starcoin_framework::coin;
    use starcoin_framework::fungible_asset::{Self, FungibleStore, Metadata};
    use starcoin_framework::object;
    use starcoin_framework::object::{ExtendRef, Object};
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::stc_util;
    use starcoin_framework::system_addresses;

    use starcoin_std::debug;
    use starcoin_std::smart_table;

    #[test_only]
    use starcoin_framework::account;
    #[test_only]
    use starcoin_framework::starcoin_coin::{Self, STC};

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
        token_stores: smart_table::SmartTable<Object<Metadata>, AssetMappingStore>
    }

    struct AssetMappingProof has key, store {
        proof_root: vector<u8>,
        anchor_height: u64,
    }

    /// Error code for invalid signer
    const EINVALID_SIGNER: u64 = 101;
    const EINVALID_NOT_PROOF: u64 = 102;
    const EINVALID_PROOF_ROOT: u64 = 102;

    /// Initializes the asset mapping pool
    /// @param framework - The framework signer
    /// @param proof_root - Initial proof root for verification
    /// @param anchor_height - Initial anchor height
    /// Verifies the framework signer and creates a new AssetMappingPool
    public fun initialize(framework: &signer) {
        assert!(
            signer::address_of(framework) == system_addresses::get_starcoin_framework(),
            error::unauthenticated(EINVALID_SIGNER)
        );
        move_to(framework, AssetMappingPool {
            token_stores: smart_table::new<Object<Metadata>, AssetMappingStore>(),
        });
    }

    /// Called by StarcoinNode after Genesis
    public entry fun initalize_proof(framework: &signer, proof_root: vector<u8>, anchor_height: u64) {
        assert!(
            signer::address_of(framework) == system_addresses::get_starcoin_framework(),
            error::unauthenticated(EINVALID_SIGNER)
        );
        move_to(framework, AssetMappingProof {
            proof_root,
            anchor_height,
        });
    }

    /// Creates a new store from a coin
    /// @param token_issuer - The token issuer signer
    /// @param coin - The coin to be stored
    /// Requirements:
    /// - Token issuer must be authorized for the given token type
    /// - Converts coin to fungible asset and stores it
    public fun create_store_from_coin<T: key>(token_issuer: &signer, coin: coin::Coin<T>) acquires AssetMappingPool {
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

    /// Assigns tokens to a recipient account with proof verification
    /// @param token_issuer - The token issuer signer
    /// @param receiper - Recipient address
    /// @param proove - Proof data for verification
    /// @param amount - Amount of tokens to assign
    /// Requirements:
    /// - Valid proof must be provided
    /// - Sufficient balance must exist
    public entry fun assign_to_account<T>(
        token_issuer: &signer,
        receiper: address,
        proove: vector<u8>,
        amount: u64
    ) acquires AssetMappingPool {
        assert!(
            exists<AssetMappingProof>(system_addresses::get_starcoin_framework()),
            error::invalid_state(EINVALID_PROOF_ROOT)
        );

        let metadata = coin::ensure_paired_metadata<T>();
        let mapping_pool = borrow_global_mut<AssetMappingPool>(signer::address_of(token_issuer));
        let mapping_store = smart_table::borrow_mut(&mut mapping_pool.token_stores, metadata);

        assert!(calculation_proof(proove, vector::empty()), error::unauthenticated(EINVALID_NOT_PROOF));

        let store_signer = object::generate_signer_for_extending(&mapping_store.extend_ref);
        fungible_asset::deposit(
            primary_fungible_store::ensure_primary_store_exists(receiper, metadata),
            fungible_asset::withdraw(&store_signer, mapping_store.fungible_store, amount)
        )
    }

    /// Computes and verifies the provided proof
    fun calculation_proof(_leaf: vector<u8>, _siblings: vector<vector<u8>>): bool {
        // TODO(BobOng): implement this function
        true
    }

    // Test function for asset mapping store creation and assignment
    // Tests
    //  Store creation from coin
    //  Balance checking
    //  Asset assignment to account
    //  Final balance verification
    #[test(framework= @starcoin_framework, alice= @0x123)]
    fun test_asset_mapping_create_store_from_coin(framework: &signer, alice: &signer) acquires AssetMappingPool {
        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | entered"));

        let amount = 10000000000;
        Self::initialize(framework);
        Self::initalize_proof(framework, vector::empty<u8>(), 0);

        // create genesis account
        account::create_account_for_test(signer::address_of(framework));

        let (burn_cap, mint_cap) = starcoin_coin::initialize_for_test(framework);
        coin::register<STC>(framework);
        starcoin_coin::mint(framework, signer::address_of(framework), amount);

        let coin = coin::withdraw<STC>(framework, amount);
        create_store_from_coin<STC>(framework, coin);
        assert!(Self::balance<STC>() == amount, 10001);

        // Assign to alice
        let alice_addr = signer::address_of(alice);
        assign_to_account<STC>(framework, alice_addr, vector::empty<u8>(), amount);
        assert!(Self::balance<STC>() == 0, 10002);

        let stc_metadata = coin::ensure_paired_metadata<STC>();
        assert!(primary_fungible_store::balance(alice_addr, stc_metadata) == amount, 10003);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);

        debug::print(&std::string::utf8(b"asset_mapping::test_asset_mapping_create_store_from_coin | exited"));
    }
}
