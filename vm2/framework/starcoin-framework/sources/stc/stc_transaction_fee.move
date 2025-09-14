/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
/// Uses aggregator_v2 for parallel execution and distributes fees across 100 genesis accounts.
module starcoin_framework::stc_transaction_fee {
    use starcoin_std::debug;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::system_addresses;
    use starcoin_framework::aggregator_v2;
    use starcoin_framework::create_signer;
    use starcoin_std::from_bcs;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// The `AutoIncrementCounter` resource holds an aggregator counter for parallel execution
    /// and tracks which genesis account to send fees to next.
    struct AutoIncrementCounter<phantom TokenType> has key {
        /// Counter that keeps incrementing to determine which genesis account to use
        counter: aggregator_v2::Aggregator<u64>,
    }

    /// Called in genesis. Sets up the needed resources to collect transaction fees using
    /// the parallel aggregator approach.
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
        aborts_if exists<AutoIncrementCounter<STC>>(signer::address_of(account));
    }

    /// publishing a wrapper of the `AutoIncrementCounter` resource under `fee_account`
    fun add_txn_fee_token<TokenType>(account: &signer) {
        move_to(
            account,
            AutoIncrementCounter<TokenType> {
                counter: aggregator_v2::create_unbounded_aggregator(),
            }
        )
    }

    spec add_txn_fee_token {
        use std::signer;
        aborts_if exists<AutoIncrementCounter<TokenType>>(signer::address_of(account));
    }

    /// Helper function to create a storage account address from predefined addresses
    fun next_storage_address<TokenType>(): address acquires AutoIncrementCounter {
        // only one reserved account which might be starcoin_framework address
        if (system_addresses::reserved_account_to() == system_addresses::reserved_account_from() + 1) {
            from_bcs::u128_to_address(system_addresses::reserved_account_from())
        } else {
            let counter_resource = borrow_global_mut<AutoIncrementCounter<TokenType>>(
                system_addresses::get_starcoin_framework()
            );

            // add/read is not atomic, but txn execution cost much more time, we can ignore contention here
            aggregator_v2::add(&mut counter_resource.counter, 1);
            let counter = (aggregator_v2::read(&counter_resource.counter) as u128);

            let range = system_addresses::reserved_account_to() - system_addresses::reserved_account_from() - 1;
            let addr_u128 = system_addresses::reserved_account_from() + (counter % range);

            let addr = from_bcs::u128_to_address(addr_u128);
            // avoiding transfer gas to framework account, which is prone to raise conflict in parallel execution
            if (addr == system_addresses::get_starcoin_framework()) {
                next_storage_address<TokenType>()
            } else {
                addr
            }
        }
    }

    /// Deposit `token` into one of the storage accounts
    public fun pay_fee<TokenType>(token: coin::Coin<TokenType>) acquires AutoIncrementCounter {
        // Get the target genesis account address
        let deposit_address = next_storage_address<TokenType>();
        
        // Deposit the fee directly to the selected genesis account
        coin::deposit(deposit_address, token);
    }

    spec pay_fee {
        use starcoin_framework::system_addresses;

        aborts_if !exists<AutoIncrementCounter<TokenType>>(system_addresses::get_starcoin_framework());
    }

    /// Collect transaction fees from all 100 genesis accounts and return total as coin.
    /// This function iterates through all genesis accounts and withdraws available fees.
    public fun distribute_transaction_fees<TokenType>(
        account: &signer,
    ): coin::Coin<TokenType> acquires AutoIncrementCounter {
        debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Entered"));

        system_addresses::assert_starcoin_framework(account);

        // Create accumulator for all collected fees
        let total_fees = coin::zero<TokenType>();
        
        let first_withdraw_address = next_storage_address<TokenType>();

        while (true) {
            let withdraw_address = next_storage_address<TokenType>();

            let balance = coin::balance<TokenType>(withdraw_address);
            if (balance > 0) {
                // Create signer for the genesis account and withdraw all funds
                let genesis_signer = create_signer::create_signer(withdraw_address);
                let withdrawn_coin = coin::withdraw<TokenType>(&genesis_signer, balance);
                coin::merge(&mut total_fees, withdrawn_coin);
            };

            if (withdraw_address == first_withdraw_address) break;
        };

        total_fees
    }

    spec distribute_transaction_fees {
        use std::signer;

        pragma verify = false;
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
    }
}
