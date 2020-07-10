address 0x1 {

module TransactionFee {
    //use 0x1::Account;
    use 0x1::Coin::{Self,Coin};
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::STC::{STC};
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

    /// Deposit `coin` into the transaction fees bucket
    public fun pay_fee<CoinType>(coin: Coin<CoinType>) acquires TransactionFee {
        let fees = borrow_global_mut<TransactionFee<CoinType>>(
            CoreAddresses::GENESIS_ACCOUNT()
        );
        Coin::deposit(&mut fees.fee, coin)
    }

    /// Distribute the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is STC, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun distribute_transaction_fees<CoinType>(
        account: &signer,
    ): Coin<CoinType> acquires TransactionFee {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        let fee_address =  CoreAddresses::GENESIS_ACCOUNT();

        // extract fees
        let txn_fees = borrow_global_mut<TransactionFee<CoinType>>(fee_address);
        let value = Coin::value<CoinType>(&txn_fees.fee);
        if (value > 0) {
            Coin::withdraw_all(&mut txn_fees.fee)
        }else {
            Coin::zero<CoinType>()
        }
    }
 }
}
