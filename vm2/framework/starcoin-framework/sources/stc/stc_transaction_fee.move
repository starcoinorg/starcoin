/// `TransactionFee` collect gas fees used by transactions in blocks temporarily.
module starcoin_framework::stc_transaction_fee {
    use starcoin_std::debug;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::system_addresses;
    use starcoin_framework::create_signer;
    use starcoin_std::from_bcs;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    native fun atomic_counter_fetch_add() : u128;

    /// Helper function to create a storage account address from predefined addresses
    fun next_storage_address<TokenType>(range_from: u128, range_to: u128): address {
        assert!(range_to > range_from, 0);
        if (range_to == range_from + 1) {
            from_bcs::u128_to_address(range_from)
        } else {
            loop {
                let counter = atomic_counter_fetch_add();

                let range = range_to - range_from - 1;
                let addr_u128 = range_from + (counter % range);

                let addr = from_bcs::u128_to_address(addr_u128);
                // avoid using the framework account address, which is prone to create conflict
                if (addr != system_addresses::get_starcoin_framework()) {
                    return addr;
                }
            }
        }
    }

    /// Deposit `token` into one of the storage accounts
    public fun pay_fee<TokenType>(token: coin::Coin<TokenType>) {
        // Get the target genesis account address
        let range_from = system_addresses::reserved_account_from();
        let range_to = system_addresses::reserved_account_to();
        let deposit_address = next_storage_address<TokenType>(range_from, range_to);
        
        // Deposit the fee directly to the selected genesis account
        coin::deposit(deposit_address, token);
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
        
        let range_from = system_addresses::reserved_account_from();
        let range_to = system_addresses::reserved_account_to();
        let first_withdraw_address = next_storage_address<TokenType>(range_from, range_to);

        while (true) {
            let withdraw_address = next_storage_address<TokenType>(range_from, range_to);

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
