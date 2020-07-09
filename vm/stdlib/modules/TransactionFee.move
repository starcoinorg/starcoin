address 0x1 {

module TransactionFee {
    use 0x1::Account;
    use 0x1::Coin::{Self,Coin};
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::STC::{Self,STC};
    use 0x1::Timestamp;

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `CoinType` that can be collected as a transaction fee.
    resource struct TransactionFee<CoinType> {
        fee: Coin<CoinType>,
    }

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(
        account: &signer,
    ) {
        assert(Timestamp::is_genesis(), 1);
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

        // accept fees in all the currencies
        add_txn_fee_currency<STC>(account);
    }

    /// publishing a wrapper of the `Preburn<CoinType>` resource under `fee_account`
    fun add_txn_fee_currency<CoinType>(
        account: &signer,
    ) {
        move_to(
            account,
            TransactionFee<CoinType> {
                fee: Coin::zero(),
            }
        )
     }

    /// Distribute the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is STC, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun distribute_transaction_fees<CoinType>(
        account: &signer,
        current_author: address,
    ) acquires TransactionFee {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        assert(Account::exists_at(current_author), 6100);
        let fee_address =  CoreAddresses::GENESIS_ACCOUNT();
        if (STC::is_stc<CoinType>()) {
            // extract fees
            let txn_fees = borrow_global_mut<TransactionFee<STC>>(fee_address);
            let value = Coin::value<STC>(&txn_fees.fee);
            if (value > 0) {
                let coins = Coin::withdraw_all<STC>(&mut txn_fees.fee);
                Account::deposit<STC>(account, current_author, coins);
            }
        } else {
            // extract fees
            let txn_fees = borrow_global_mut<TransactionFee<CoinType>>(fee_address);
            let value = Coin::value<CoinType>(&txn_fees.fee);
            if (value > 0) {
                let coins = Coin::withdraw_all(&mut txn_fees.fee);
                Account::deposit<CoinType>(account, current_author, coins);
            }
        }
    }
}
}
