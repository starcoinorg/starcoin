/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Then they are distributed in `TransactionManager`.
module starcoin_framework::transaction_fee {
    use std::error;
    use std::option;
    use std::option::Option;
    use std::vector;
    use starcoin_framework::fungible_asset;

    use starcoin_framework::object::{Self, Object, DeriveRef};
    use starcoin_framework::fungible_asset::{FungibleStore, create_store, FungibleAsset, Metadata};
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::system_addresses::{Self, get_starcoin_framework};

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    const ETXN_FEE_STC_METADATA_NOT_INITIALIZED: u64 = 1;
    const ETXN_FEE_STORE_NOT_INITIALIZED: u64 = 2;

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `TokenType` that can be collected as a transaction fee.
    struct TransactionFee has key {
        fee_stores: vector<Object<FungibleStore>>,
        derive_ref: DeriveRef,
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
        move_to(framework, TransactionFee {
            fee_stores,
            derive_ref: object::generate_derive_ref(&constr_ref),
        });
    }

    spec initialize {
        pragma verify = false;
    }

    fun metadat_to_asset_store(metadata: Object<Metadata>): Option<Object<FungibleStore>> acquires TransactionFee {
        assert!(exists<TransactionFee>(get_starcoin_framework()), error::invalid_state(ETXN_FEE_STORE_NOT_INITIALIZED));

        let fee = borrow_global_mut<TransactionFee>(get_starcoin_framework());
        let fee_len = vector::length(&fee.fee_stores);
        assert!(fee_len > 0, error::invalid_state(ETXN_FEE_STORE_NOT_INITIALIZED));

        let idx: u64 = 0;
        while (idx < fee_len) {
            let store = vector::borrow(&fee.fee_stores, idx);
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
    public fun pay_fee_fa(fa: FungibleAsset) acquires TransactionFee {
        let store_opt = metadat_to_asset_store(fungible_asset::metadata_from_asset(&fa));
        assert!(option::is_some(&store_opt), error::invalid_state(ETXN_FEE_STORE_NOT_INITIALIZED));

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
        _account: &signer,
    ): coin::Coin<TokenType> {
        // debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Entered"));
        //
        // let fee_address = system_addresses::get_starcoin_framework();
        // system_addresses::assert_starcoin_framework(account);
        //
        // // extract fees
        // let txn_fees = borrow_global_mut<TransactionFee<TokenType>>(fee_address);
        // let value = coin::value<TokenType>(&txn_fees.fee);
        //
        // if (value > 0) {
        //     debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Exit with value: "));
        //     debug::print(&value);
        //     coin::extract(&mut txn_fees.fee, value)
        // } else {
        //     debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Exit with zero"));
        //     coin::zero<TokenType>()
        // }
        coin::zero()
    }

    spec distribute_transaction_fees {
        pragma verify = false;
    }
}
