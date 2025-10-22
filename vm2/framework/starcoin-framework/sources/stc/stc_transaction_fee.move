/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Then they are distributed in `TransactionManager`.
module starcoin_framework::stc_transaction_fee {
    use starcoin_std::debug;
    use std::signer;
    use std::vector;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::system_addresses;

    friend starcoin_framework::stc_block;

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
        add_txn_fee_token<STC>(account);
    }

    spec initialize {
        use std::signer;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<TransactionFee<STC>>(signer::address_of(account));
    }

    /// publishing a wrapper of the `Preburn<TokenType>` resource under `fee_account`
    fun add_txn_fee_token<TokenType>(account: &signer) {
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
    public fun pay_fee<TokenType>(account: &signer, token: coin::Coin<TokenType>) acquires TransactionFee {
        if (!exists<TransactionFee<TokenType>>(signer::address_of(account))) {
            move_to(
                account,
                TransactionFee<TokenType> { fee: coin::zero() }
            );
        };
        
        let addr = signer::address_of(account);
        let txn_fees = borrow_global_mut<TransactionFee<TokenType>>(
            addr
        );
        coin::merge(&mut txn_fees.fee, token)
    }

    fun inner_distribute_transaction_fees<TokenType>(
        addr: address,
    ): coin::Coin<TokenType> acquires TransactionFee {
        debug::print(&std::string::utf8(b"stc_transaction_fee::inner_distribute_transaction_fees | Entered"));

        // extract fees
        let txn_fees = borrow_global_mut<TransactionFee<TokenType>>(addr);
        let value = coin::value<TokenType>(&txn_fees.fee);

        if (value > 0) {
            debug::print(&std::string::utf8(b"stc_transaction_fee::inner_distribute_transaction_fees | Exit with value: "));
            debug::print(&value);
            coin::extract(&mut txn_fees.fee, value)
        } else {
            debug::print(&std::string::utf8(b"stc_transaction_fee::inner_distribute_transaction_fees | Exit with zero"));
            coin::zero<TokenType>()
        }
    }

    public fun merge_fee_to_framework_account(account: &signer, senders: vector<address>) acquires TransactionFee {
        system_addresses::assert_starcoin_framework(account);

        let framework_address = system_addresses::get_starcoin_framework();
        let len = vector::length(&senders);
        for (i in 0..len) {
            let addr = *vector::borrow(&senders, i);
            if (addr != framework_address && exists<TransactionFee<STC>>(addr)) {
                let token = inner_distribute_transaction_fees<STC>(addr);
                pay_fee<STC>(account, token);
            }
        }
    }

    /// Distribute the transaction fees collected in the `TokenType` token.
    /// If the `TokenType` is STC, it unpacks the token and preburns the
    /// underlying fiat.
    public fun distribute_transaction_fees<TokenType>(
        account: &signer,
    ): coin::Coin<TokenType> acquires TransactionFee {
        system_addresses::assert_starcoin_framework(account);
        inner_distribute_transaction_fees<TokenType>(signer::address_of(account))
    }

    spec distribute_transaction_fees {
        use std::signer;

        pragma verify = false;
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<TransactionFee<TokenType>>(system_addresses::get_starcoin_framework());
    }
}
