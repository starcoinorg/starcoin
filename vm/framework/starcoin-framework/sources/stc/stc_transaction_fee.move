/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Then they are distributed in `TransactionManager`.
module starcoin_framework::stc_transaction_fee {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::system_addresses;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `TokenType` that can be collected as a transaction fee.
    struct TransactionFee<phantom TokenType> has key {
        fee: coin::Coin<TokenType>,
    }

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(account: &signer) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(account);

        // accept fees in all the currencies
        add_txn_fee_token<coin::Coin<STC>>(account);
    }

    spec initialize {
        use std::signer;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<TransactionFee<STC>>(signer::address_of(account));
    }

    /// publishing a wrapper of the `Preburn<TokenType>` resource under `fee_account`
    fun add_txn_fee_token<TokenType: store>(account: &signer) {
        move_to(
            account,
            TransactionFee<TokenType> {
                fee: coin::zero(),
            }
        )
    }

    spec add_txn_fee_token {
        use std::signer;
        aborts_if exists<TransactionFee<TokenType>>(signer::address_of(account));
    }

    /// Deposit `token` into the transaction fees bucket
    public fun pay_fee<TokenType: store>(token: coin::Coin<TokenType>) acquires TransactionFee {
        let txn_fees = borrow_global_mut<TransactionFee<TokenType>>(
            system_addresses::get_starcoin_framework()
        );
        coin::merge(&mut txn_fees.fee, token)
    }

    spec pay_fee {
        use starcoin_framework::system_addresses;

        aborts_if !exists<TransactionFee<TokenType>>(system_addresses::get_starcoin_framework());
        aborts_if global<TransactionFee<TokenType>>(
            system_addresses::get_starcoin_framework()
        ).fee.value + token.value > max_u128();
    }

    /// Distribute the transaction fees collected in the `TokenType` token.
    /// If the `TokenType` is STC, it unpacks the token and preburns the
    /// underlying fiat.
    public fun distribute_transaction_fees<TokenType: store>(
        account: &signer,
    ): coin::Coin<TokenType> acquires TransactionFee {
        let fee_address = system_addresses::get_starcoin_framework();
        system_addresses::assert_starcoin_framework(account);

        // extract fees
        let txn_fees = borrow_global_mut<TransactionFee<TokenType>>(fee_address);
        let value = coin::value<TokenType>(&txn_fees.fee);
        if (value > 0) {
            coin::extract(&mut txn_fees.fee, value)
        }else {
            coin::zero<TokenType>()
        }
    }

    spec distribute_transaction_fees {
        pragma verify = false;
        //        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        //        aborts_if !exists<TransactionFee<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
    }
}
