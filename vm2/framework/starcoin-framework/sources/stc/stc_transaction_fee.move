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

    native fun address_to_u128(addr: address): u128;

    /// Deposit `token` into one of the storage accounts
    ///  txn sender is used as hasher to uniquely locate a reserved account to receive the gas
    public fun pay_fee<TokenType>(txn_sender: address, token: coin::Coin<TokenType>) {
        // Get the target reserved account address
        let range_from = system_addresses::reserved_account_from();
        let range_to = system_addresses::reserved_account_to();
        let span = range_to - range_from;
        let addr_u128 = range_from + (address_to_u128(txn_sender) % span); 
        let addr = from_bcs::u128_to_address(addr_u128);
        
        // Deposit the fee directly to the selected genesis account
        coin::deposit(addr, token);
    }

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
        for (addr_u128 in range_from..range_to) {
            let withdraw_address = from_bcs::u128_to_address(addr_u128);
            let balance = coin::balance<TokenType>(withdraw_address);
            if (balance > 0) {
                // Create signer for the genesis account and withdraw all funds
                let genesis_signer = create_signer::create_signer(withdraw_address);
                let withdrawn_coin = coin::withdraw<TokenType>(&genesis_signer, balance);
                coin::merge(&mut total_fees, withdrawn_coin);
            };
        };

        total_fees
    }

    spec distribute_transaction_fees {
        use std::signer;

        pragma verify = false;
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
    }
}
