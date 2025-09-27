/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Then they are distributed in `TransactionManager`.
module starcoin_framework::transaction_fee {
    use std::error;
    use std::option;
    use std::option::Option;
    use std::vector;

    use starcoin_framework::object::{Self, Object};
    use starcoin_framework::fungible_asset::{Self, FungibleStore, create_store, FungibleAsset, Metadata};
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::system_addresses::{Self, get_starcoin_framework};

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    const ETXN_FEE_STC_METADATA_NOT_INITIALIZED: u64 = 1;
    const ETXN_FEE_FA_STORE_NOT_INITIALIZED: u64 = 2;
    const ETXN_FEE_FA_METADATA_NOT_INITIALIZED: u64 = 3;

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `TokenType` that can be collected as a transaction fee.
    struct TransactionFeePod has key {
        fee_stores: vector<Object<FungibleStore>>,
        tranfer_ref: fungible_asset::TransferRef,
    }

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(framework: &signer) {
        system_addresses::assert_starcoin_framework(framework);

        let constr_ref = object::create_named_object(framework, b"transaction_fee");
        let stc_metadata = coin::paired_metadata<STC>();
        assert!(option::is_some(&stc_metadata), error::invalid_state(ETXN_FEE_STC_METADATA_NOT_INITIALIZED));

        let fee_fa_store = create_store(&constr_ref, option::destroy_some(stc_metadata));
        let fee_stores = vector::empty<Object<FungibleStore>>();

        vector::push_back(&mut fee_stores, fee_fa_store);
        move_to(framework, TransactionFeePod {
            fee_stores,
            tranfer_ref: fungible_asset::generate_transfer_ref(&constr_ref),
        });
    }

    spec initialize {
        pragma verify = false;
    }

    fun metadata_to_asset_store(
        fee_stores: &vector<Object<FungibleStore>>,
        metadata: Object<Metadata>
    ): Option<Object<FungibleStore>> {
        let fee_len = vector::length(fee_stores);
        assert!(fee_len > 0, error::invalid_state(ETXN_FEE_FA_STORE_NOT_INITIALIZED));

        let idx: u64 = 0;
        while (idx < fee_len) {
            let store = vector::borrow(fee_stores, idx);
            if (fungible_asset::store_metadata(*store) == metadata) {
                return option::some(*store)
            };
        };
        option::none()
    }

    /// Deposit `token` into the transaction fees bucket
    public fun pay_fee<TokenType>(_coin: coin::Coin<TokenType>) {
        coin::destroy_zero(_coin);
    }

    /// Deposit `token` into the transaction fees bucket
    public fun pay_fee_fa(fa: FungibleAsset) acquires TransactionFeePod {
        assert!(exists<TransactionFeePod>(get_starcoin_framework()), error::invalid_state(
            ETXN_FEE_FA_STORE_NOT_INITIALIZED
        ));

        let fee_pod = borrow_global_mut<TransactionFeePod>(get_starcoin_framework());
        let store_opt = metadata_to_asset_store(&fee_pod.fee_stores, fungible_asset::metadata_from_asset(&fa));
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
        account: &signer,
    ): FungibleAsset acquires TransactionFeePod {
        system_addresses::assert_starcoin_framework(account);

        assert!(exists<TransactionFeePod>(get_starcoin_framework()), error::invalid_state(
            ETXN_FEE_FA_STORE_NOT_INITIALIZED
        ));

        let fee_pod = borrow_global_mut<TransactionFeePod>(get_starcoin_framework());

        let metadata = coin::paired_metadata<TokenType>();
        assert!(option::is_some(&metadata), error::invalid_state(ETXN_FEE_FA_METADATA_NOT_INITIALIZED));
        let fa_store_opt =
            metadata_to_asset_store(&fee_pod.fee_stores, option::destroy_some(metadata));

        assert!(option::is_some(&fa_store_opt), error::invalid_state(ETXN_FEE_FA_STORE_NOT_INITIALIZED));

        let fa_store = option::destroy_some(fa_store_opt);
        let all_asset_balance = fungible_asset::balance(fa_store);

        fungible_asset::withdraw_with_ref(&fee_pod.tranfer_ref, fa_store, all_asset_balance)
    }

    spec distribute_transaction_fees {
        pragma verify = false;
    }

    // #[test(framework = @0x1)]
    // fun test_intialize(framework: &signer) {
    //     Self::initialize(framework)
    // }
}
