/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Then they are distributed in `TransactionManager`.
module starcoin_framework::transaction_fee {
    friend starcoin_framework::stc_block;
    friend starcoin_framework::stc_genesis;
    friend starcoin_framework::stc_transaction_validation;

    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::string::{Self, utf8};
    use std::vector;

    use starcoin_framework::coin;
    use starcoin_framework::coin::{BurnCapability, MintCapability};
    use starcoin_framework::create_signer::create_signer;
    use starcoin_framework::fungible_asset::{Self, FungibleAsset, FungibleStore, Metadata};
    use starcoin_framework::object::{Self, Object};
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::system_addresses::{Self, get_starcoin_framework};
    use starcoin_std::debug;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    const ETXN_FEE_POD_NOT_INITIALIZED: u64 = 1;
    const ETXN_FEE_POD_HAS_INITIALIZED: u64 = 2;
    const ETXN_FEE_FA_METADATA_NOT_FOUND: u64 = 3;
    const ETXN_FEE_FA_STORE_NOT_FOUND: u64 = 4;
    const ETXN_FEE_STORES_IS_EMPTY: u64 = 5;
    const ETXN_FEE_CAPS_ALREADY_INIT: u64 = 6;

    #[resource_group_member(group = starcoin_framework::object::ObjectGroup)]
    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `TokenType` that can be collected as a transaction fee.
    struct TransactionFeePod has key {
        fee_stores: vector<Object<FungibleStore>>,
    }

    /// Mint/Burn capabilities for fee accounting.
    struct FeeCapStore has key {
        mint_cap: MintCapability<starcoin_coin::STC>,
        burn_cap: BurnCapability<starcoin_coin::STC>,
    }

    public(friend) fun store_fee_caps(
        framework: &signer,
        burn_cap: BurnCapability<starcoin_coin::STC>,
        mint_cap: MintCapability<starcoin_coin::STC>,
    ) {
        system_addresses::assert_starcoin_framework(framework);
        assert!(
            !exists<FeeCapStore>(signer::address_of(framework)),
            error::already_exists(ETXN_FEE_CAPS_ALREADY_INIT)
        );
        move_to(framework, FeeCapStore { mint_cap, burn_cap });
    }

    public(friend) fun burn_fee(fa: FungibleAsset) acquires FeeCapStore {
        let cap = &borrow_global<FeeCapStore>(get_starcoin_framework()).burn_cap;
        coin::burn_fungible_asset_with_cap<starcoin_coin::STC>(fa, cap);
    }

    /// Burn transaction fee directly from the sender's primary store without emitting events.
    public(friend) fun burn_fee_from(account: address, fee: u64) acquires FeeCapStore {
        if (fee == 0) {
            return
        };
        let cap = &borrow_global<FeeCapStore>(get_starcoin_framework()).burn_cap;
        coin::burn_from_for_gas<starcoin_coin::STC>(account, fee, cap);
    }

    public(friend) fun mint_fee(amount: u64): FungibleAsset acquires FeeCapStore {
        let cap = &borrow_global<FeeCapStore>(get_starcoin_framework()).mint_cap;
        let (mint_ref, receipt) = coin::get_paired_mint_ref(cap);
        let fa = fungible_asset::mint(&mint_ref, amount);
        coin::return_paired_mint_ref(mint_ref, receipt);
        fa
    }

    public fun initialize(framework: &signer) {
        assert!(
            !exists<TransactionFeePod>(signer::address_of(framework)),
            error::not_found(ETXN_FEE_POD_NOT_INITIALIZED)
        );

        let fee_stores = vector::empty();
        vector::push_back(
            &mut fee_stores,
            Self::inner_create_fa_store(framework, starcoin_coin::get_stc_fa_metadata())
        );
        move_to(framework, TransactionFeePod {
            fee_stores
        });
    }

    /// Deposit `token` into the transaction fees bucket
    public fun pay_fee(account: &signer, fa: FungibleAsset) acquires TransactionFeePod {
        let account_addr = signer::address_of(account);

        let fa_store = if (exists<TransactionFeePod>(account_addr)) {
            let fee_pod = borrow_global_mut<TransactionFeePod>(get_starcoin_framework());
            let store_opt = find_asset_store_with_metadata(
                &fee_pod.fee_stores,
                fungible_asset::metadata_from_asset(&fa)
            );
            if (option::is_some(&store_opt)) {
                option::destroy_some(store_opt)
            } else {
                let fa_store = Self::inner_create_fa_store(account, fungible_asset::metadata_from_asset(&fa));
                vector::push_back(&mut fee_pod.fee_stores, fa_store);
                fa_store
            }
        } else {
            let fa_store = Self::inner_create_fa_store(account, starcoin_coin::get_stc_fa_metadata());
            let fee_stores = vector::empty();
            vector::push_back(&mut fee_stores, fa_store);
            move_to(account, TransactionFeePod {
                fee_stores
            });
            fa_store
        };
        fungible_asset::deposit_internal_no_events(object::object_address(&fa_store), fa);
    }

    spec pay_fee {
        pragma verify = false;
    }

    public fun merge_fee_to_framework_account(
        framework: &signer,
        payer_addresses: vector<address>
    ) acquires TransactionFeePod {
        system_addresses::assert_starcoin_framework(framework);
        debug::print(&utf8(b"transaction_fee::merge_fee_to_framework_account | Entered, payer addresses: "));
        debug::print(&payer_addresses);

        let framework_address = system_addresses::get_starcoin_framework();
        let len = vector::length(&payer_addresses);
        for (i in 0 .. len) {
            let addr = *vector::borrow(&payer_addresses, i);
            if (addr != framework_address && exists<TransactionFeePod>(addr)) {
                let transaction_fee_pod = borrow_global<TransactionFeePod>(addr);
                let stc_metadata = starcoin_coin::get_stc_fa_metadata();
                let fa_store = Self::find_asset_store_with_metadata(
                    &transaction_fee_pod.fee_stores,
                    stc_metadata
                );
                assert!(option::is_some(&fa_store), error::not_found(ETXN_FEE_FA_STORE_NOT_FOUND));
                let fa = Self::withdraw_account_transaction_fees(&create_signer(addr), stc_metadata);
                pay_fee(framework, fa);
            }
        };

        debug::print(&utf8(b"transaction_fee::merge_fee_to_framework_account | Exited"));
    }

    /// Distribute the transaction fees collected in the `TokenType` token.
    /// If the `TokenType` is STC, it unpacks the token and preburns the
    /// underlying fiat.
    public fun withdraw_account_transaction_fees(
        account: &signer,
        metadata: Object<Metadata>
    ): FungibleAsset acquires TransactionFeePod {
        debug::print(&std::string::utf8(b"transaction_fee::withdraw_account_transaction_fees | Entered"));

        let account_addr = signer::address_of(account);
        assert!(exists<TransactionFeePod>(account_addr), error::not_found(ETXN_FEE_POD_NOT_INITIALIZED));

        let fee_pod = borrow_global_mut<TransactionFeePod>(account_addr);
        let fa_store_opt = find_asset_store_with_metadata(&fee_pod.fee_stores, metadata);
        assert!(option::is_some(&fa_store_opt), error::invalid_state(ETXN_FEE_FA_STORE_NOT_FOUND));

        let fa_store = option::destroy_some(fa_store_opt);
        let all_asset_balance = fungible_asset::balance(fa_store);

        let ret = fungible_asset::withdraw(account, fa_store, all_asset_balance);

        debug::print(&std::string::utf8(b"transaction_fee::withdraw_account_transaction_fees | Exited"));

        ret
    }

    fun inner_create_fa_store(account: &signer, metadata: Object<Metadata>): Object<FungibleStore> {
        let fa_store_seed = Self::get_fa_store_seed(metadata);
        let construct_addr = object::create_object_address(&signer::address_of(account), fa_store_seed);
        let fa_store = if (object::object_exists<FungibleStore>(construct_addr)) {
            object::address_to_object<FungibleStore>(construct_addr)
        } else {
            let construct_ref = object::create_named_object(account, fa_store_seed);
            fungible_asset::create_store(&construct_ref, metadata)
        };
        fa_store
    }

    fun find_asset_store_with_metadata(
        fee_stores: &vector<Object<FungibleStore>>,
        target_metadata: Object<Metadata>
    ): Option<Object<FungibleStore>> {
        let fee_len = vector::length(fee_stores);
        assert!(fee_len > 0, error::invalid_state(ETXN_FEE_STORES_IS_EMPTY));

        let idx: u64 = 0;
        while (idx < fee_len) {
            let store = vector::borrow(fee_stores, idx);
            debug::print(store);
            let store_metadata = fungible_asset::store_metadata(*store);
            if (store_metadata == target_metadata) {
                return option::some(*store)
            };
            idx = idx + 1;
        };
        option::none()
    }

    spec withdraw_account_transaction_fees {
        pragma verify = false;
    }

    fun get_fa_store_seed(metadata: Object<Metadata>): vector<u8> {
        let seed = vector::empty<u8>();
        vector::append(&mut seed, b"transaction_fee");
        vector::push_back(&mut seed, 0xFE);
        vector::append(&mut seed, *string::bytes(&fungible_asset::name(metadata)));
        seed
    }

    #[test(framework = @0x1, alice = @0x123, bob = @0x456)]
    fun test_txn_fee_basic_flow(framework: &signer, alice: &signer, bob: &signer) acquires TransactionFeePod {
        use starcoin_framework::starcoin_account;
        use starcoin_framework::starcoin_coin;
        use starcoin_framework::primary_fungible_store;
        use std::signer;

        starcoin_coin::ensure_initialized_with_stc_fa_metadata_for_test();
        Self::initialize(framework);

        let stc_metadata = starcoin_coin::get_stc_fa_metadata();
        let amount: u64 = 100000000;
        let pay_addresses = vector::empty();

        // Pay fee for alice
        let alice_addr = signer::address_of(alice);
        starcoin_account::create_account(alice_addr);
        Self::pay_fee(alice, starcoin_coin::mint_stc_fa_for_test(amount));
        vector::push_back(&mut pay_addresses, alice_addr);

        // Pay fee for bob
        let bob_addr = signer::address_of(bob);
        starcoin_account::create_account(bob_addr);
        Self::pay_fee(alice, starcoin_coin::mint_stc_fa_for_test(amount));
        vector::push_back(&mut pay_addresses, bob_addr);

        Self::merge_fee_to_framework_account(framework, pay_addresses);

        let distributed_fa = Self::withdraw_account_transaction_fees(framework, stc_metadata);
        assert!(fungible_asset::amount(&distributed_fa) == amount * 2, 1);

        primary_fungible_store::deposit(signer::address_of(framework), distributed_fa);
    }

    #[test(framework = @0x1)]
    fun test_fee_burn_and_mint(framework: &signer) acquires TransactionFeePod, FeeCapStore {
        use starcoin_framework::fungible_asset;
        use starcoin_framework::starcoin_coin;

        let (burn_cap, mint_cap) = starcoin_coin::initialize_for_test(framework);
        Self::store_fee_caps(framework, burn_cap, mint_cap);
        Self::initialize(framework);

        let burn_asset = starcoin_coin::mint_stc_fa_for_test(10);
        Self::burn_fee(burn_asset);

        let total_fee = 1000;
        let minted_fee = Self::mint_fee(total_fee);
        Self::pay_fee(framework, minted_fee);

        let fee = Self::withdraw_account_transaction_fees(
            framework,
            starcoin_coin::get_stc_fa_metadata(),
        );
        assert!(fungible_asset::amount(&fee) == total_fee, 1002);
        Self::burn_fee(fee);
    }
}
