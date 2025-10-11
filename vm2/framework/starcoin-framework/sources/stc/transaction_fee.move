/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Then they are distributed in `TransactionManager`.
module starcoin_framework::transaction_fee {
    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;

    use starcoin_framework::coin;
    use starcoin_framework::create_signer::create_signer;
    use starcoin_framework::fungible_asset::{Self, create_store, FungibleAsset, FungibleStore, Metadata};
    use starcoin_framework::object::{Self, Object};
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::system_addresses::{Self, get_starcoin_framework};
    use starcoin_std::debug;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    const ETXN_FEE_STC_METADATA_NOT_INITIALIZED: u64 = 1;
    const ETXN_FEE_FA_STORE_NOT_INITIALIZED: u64 = 2;
    const ETXN_FEE_FA_METADATA_NOT_INITIALIZED: u64 = 3;
    const ETXN_FEE_FA_STORE_NOT_FOUND: u64 = 4;

    #[resource_group_member(group = starcoin_framework::object::ObjectGroup)]
    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `TokenType` that can be collected as a transaction fee.
    struct TransactionFeePod has key {
        fee_stores: vector<Object<FungibleStore>>,
        owner_address: address,
    }

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(framework: &signer) {
        debug::print(&std::string::utf8(b"transaction_fee::initialize | Entered"));
        system_addresses::assert_starcoin_framework(framework);

        let constructor_ref = object::create_named_object(framework, b"txn_fee");
        let stc_metadata_opt = coin::paired_metadata<STC>();
        assert!(option::is_some(&stc_metadata_opt), error::invalid_state(ETXN_FEE_STC_METADATA_NOT_INITIALIZED));

        let stc_metadata = option::destroy_some(stc_metadata_opt);
        let fee_fa_store = create_store(&constructor_ref, stc_metadata);
        assert!(
            fungible_asset::store_exists(object::address_from_constructor_ref(&constructor_ref)),
            error::invalid_state(ETXN_FEE_STC_METADATA_NOT_INITIALIZED)
        );

        debug::print(&constructor_ref);
        debug::print(&fee_fa_store);

        let fee_stores = vector::empty<Object<FungibleStore>>();
        vector::push_back(&mut fee_stores, fee_fa_store);

        let owner_address = object::address_from_constructor_ref(&constructor_ref);
        move_to(
            framework,
            TransactionFeePod { fee_stores, owner_address }
        );
        debug::print(&std::string::utf8(b"transaction_fee::initialize | Exited"));
    }

    spec initialize {
        pragma verify = false;
    }

    /// Deposit `token` into the transaction fees bucket
    public fun pay_fee(fa: FungibleAsset) acquires TransactionFeePod {
        assert!(exists<TransactionFeePod>(get_starcoin_framework()), error::invalid_state(
            ETXN_FEE_FA_STORE_NOT_INITIALIZED
        ));

        let fee_pod = borrow_global_mut<TransactionFeePod>(get_starcoin_framework());
        let store_opt = find_asset_store_with_metadata(
            &fee_pod.fee_stores,
            fungible_asset::metadata_from_asset(&fa)
        );
        assert!(option::is_some(&store_opt), error::invalid_state(ETXN_FEE_FA_STORE_NOT_INITIALIZED));

        let store = option::destroy_some(store_opt);
        fungible_asset::deposit(store, fa);
    }

    spec pay_fee {
        pragma verify = false;
    }

    /// Distribute the transaction fees collected in the `TokenType` token.
    /// If the `TokenType` is STC, it unpacks the token and preburns the
    /// underlying fiat.
    public fun distribute_transaction_fees<TokenType>(
        framework: &signer,
    ): FungibleAsset acquires TransactionFeePod {
        debug::print(&std::string::utf8(b"transaction_fee::distribute_transaction_fees | Entered"));

        system_addresses::assert_starcoin_framework(framework);
        let framework_addr = signer::address_of(framework);
        assert!(exists<TransactionFeePod>(framework_addr), error::not_found(ETXN_FEE_FA_STORE_NOT_INITIALIZED));

        let fee_pod = borrow_global_mut<TransactionFeePod>(framework_addr);
        let coin_metadata_opt = coin::paired_metadata<TokenType>();
        assert!(option::is_some(&coin_metadata_opt), error::invalid_state(ETXN_FEE_FA_METADATA_NOT_INITIALIZED));

        let coin_metatdata = option::destroy_some(coin_metadata_opt);
        let fa_store_opt = find_asset_store_with_metadata(&fee_pod.fee_stores, coin_metatdata);
        assert!(option::is_some(&fa_store_opt), error::invalid_state(ETXN_FEE_FA_STORE_NOT_FOUND));

        let fa_store = option::destroy_some(fa_store_opt);
        let all_asset_balance = fungible_asset::balance(fa_store);

        let txn_fee_signer = create_signer(fee_pod.owner_address);
        let ret = fungible_asset::withdraw(&txn_fee_signer, fa_store, all_asset_balance);

        debug::print(&std::string::utf8(b"transaction_fee::distribute_transaction_fees | Exited"));

        ret
    }


    fun find_asset_store_with_metadata(
        fee_stores: &vector<Object<FungibleStore>>,
        target_metadata: Object<Metadata>
    ): Option<Object<FungibleStore>> {
        let fee_len = vector::length(fee_stores);
        assert!(fee_len > 0, error::invalid_state(ETXN_FEE_FA_STORE_NOT_INITIALIZED));

        let idx: u64 = 0;
        while (idx < fee_len) {
            let store = vector::borrow(fee_stores, idx);
            debug::print(store);
            let store_metadata = fungible_asset::store_metadata(*store);

            debug::print(&store_metadata);
            debug::print(&target_metadata);

            if (store_metadata == target_metadata) {
                return option::some(*store)
            };
            idx = idx + 1;
        };
        option::none()
    }

    spec distribute_transaction_fees {
        pragma verify = false;
    }

    #[test(framework = @0x1, alice = @0x123)]
    fun test_txn_fee_basic_flow(framework: &signer, alice: &signer) acquires TransactionFeePod {
        use starcoin_framework::starcoin_account;
        use starcoin_framework::starcoin_coin;
        use starcoin_framework::primary_fungible_store;
        use starcoin_std::debug;
        use std::string::utf8;
        use std::signer;

        starcoin_coin::ensure_initialized_with_stc_fa_metadata_for_test();
        Self::initialize(framework);

        let minted_fa = starcoin_coin::mint_stc_fa_for_test(100000000);
        let minted_fa_aount = fungible_asset::amount(&minted_fa);
        Self::pay_fee(minted_fa);

        let distributed_fa = Self::distribute_transaction_fees<STC>(framework);
        assert!(fungible_asset::amount(&distributed_fa) == minted_fa_aount, 1);

        let alice_addr = signer::address_of(alice);
        starcoin_account::create_account(alice_addr);

        debug::print(&utf8(b"transaction_fee::test_txn_fee_basic_flow | after starcoin_account::create_account"));
        primary_fungible_store::deposit(alice_addr, distributed_fa);

        debug::print(&utf8(b"transaction_fee::test_txn_fee_basic_flow | exited"));
    }
}
