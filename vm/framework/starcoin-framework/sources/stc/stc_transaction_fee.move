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
    use std::vector;

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

    /// Helper function to create a genesis account address from index (0-99)
    fun get_genesis_account_address(index: u64): address {
        // Create a 32-byte address for genesis account (0x0b + index)
        let addr_value = 0x0b + index;
        let addr_bytes = vector::empty<u8>();
        
        // Add 31 zero bytes
        let j = 0;
        while (j < 31) {
            vector::push_back(&mut addr_bytes, 0u8);
            j = j + 1;
        };
        
        // Add the address value as the last byte
        vector::push_back(&mut addr_bytes, (addr_value as u8));
        
        from_bcs::to_address(addr_bytes)
    }

    /// Deposit `token` into one of the 100 genesis accounts based on counter
    public fun pay_fee<TokenType>(token: coin::Coin<TokenType>) acquires AutoIncrementCounter {
        let counter_resource = borrow_global_mut<AutoIncrementCounter<TokenType>>(
            system_addresses::get_starcoin_framework()
        );
        
        // Increment counter and get which genesis account to use
        aggregator_v2::add(&mut counter_resource.counter, 1);
        let counter_value = aggregator_v2::read(&counter_resource.counter);
        let genesis_account_index = counter_value % 100;
        
        // Get the target genesis account address
        let target_address = get_genesis_account_address(genesis_account_index);
        
        // Deposit the fee directly to the selected genesis account
        coin::deposit(target_address, token);
    }

    spec pay_fee {
        use starcoin_framework::system_addresses;

        aborts_if !exists<AutoIncrementCounter<TokenType>>(system_addresses::get_starcoin_framework());
    }

    /// Collect transaction fees from all 100 genesis accounts and return total as coin.
    /// This function iterates through all genesis accounts and withdraws available fees.
    public fun distribute_transaction_fees<TokenType>(
        account: &signer,
    ): coin::Coin<TokenType> {
        debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Entered"));

        system_addresses::assert_starcoin_framework(account);

        // Create accumulator for all collected fees
        let total_fees = coin::zero<TokenType>();
        
        // Iterate through all 100 genesis accounts and collect their fees
        let i = 0;
        while (i < 100) {
            let genesis_address = get_genesis_account_address(i);
            
            // Check if the genesis account has any balance
            if (coin::balance<TokenType>(genesis_address) > 0) {
                let account_balance = coin::balance<TokenType>(genesis_address);
                debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Collecting from genesis account: "));
                debug::print(&i);
                debug::print(&std::string::utf8(b" with balance: "));
                debug::print(&account_balance);
                
                // Create signer for the genesis account and withdraw all funds
                let genesis_signer = create_signer::create_signer(genesis_address);
                let withdrawn_coin = coin::withdraw<TokenType>(&genesis_signer, account_balance);
                coin::merge(&mut total_fees, withdrawn_coin);
            };
            
            i = i + 1;
        };

        let total_value = coin::value<TokenType>(&total_fees);
        if (total_value > 0) {
            debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Exit with total value: "));
            debug::print(&total_value);
        } else {
            debug::print(&std::string::utf8(b"stc_block::distribute_transaction_fees | Exit with zero"));
        };

        total_fees
    }

    spec distribute_transaction_fees {
        use std::signer;

        pragma verify = false;
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
    }
}
