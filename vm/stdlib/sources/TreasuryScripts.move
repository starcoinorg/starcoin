address StarcoinFramework {
module TreasuryScripts {
    use StarcoinFramework::Treasury;
    use StarcoinFramework::Account;
    use StarcoinFramework::Offer;
    use StarcoinFramework::TreasuryWithdrawDaoProposal;

    public(script) fun withdraw_and_split_lt_withdraw_cap<TokenT: store>(
        signer: signer,
        for_address: address,
        amount: u128,
        lock_period: u64,
    ) {
        // 1. take cap: LinearWithdrawCapability<TokenT>
        let cap = Treasury::remove_linear_withdraw_capability<TokenT>(&signer);

        // 2. withdraw token and split
        let (tokens, new_cap) = Treasury::split_linear_withdraw_cap(&mut cap, amount);

        // 3. deposit
        Account::deposit_to_self(&signer, tokens);

        // 4. put or destroy key
        if (Treasury::is_empty_linear_withdraw_capability(&cap)) {
            Treasury::destroy_linear_withdraw_capability(cap);
        } else {
            Treasury::add_linear_withdraw_capability(&signer, cap);
        };

        // 5. offer
        Offer::create(&signer, new_cap, for_address, lock_period);
    }

    spec withdraw_and_split_lt_withdraw_cap {
        pragma verify = false;
    }

    public(script) fun withdraw_token_with_linear_withdraw_capability<TokenT: store>(
        signer: signer,
    ) {
        // 1. take cap
        let cap = Treasury::remove_linear_withdraw_capability<TokenT>(&signer);

        // 2. withdraw token
        let tokens = Treasury::withdraw_with_linear_capability(&mut cap);

        // 3. deposit
        Account::deposit_to_self(&signer, tokens);

        // 4. put or destroy key
        if (Treasury::is_empty_linear_withdraw_capability(&cap)) {
            Treasury::destroy_linear_withdraw_capability(cap);
        } else {
            Treasury::add_linear_withdraw_capability(&signer, cap);
        };
    }

    spec withdraw_token_with_linear_withdraw_capability {
        pragma verify = false;
    }

    public(script) fun propose_withdraw<TokenT: copy + drop + store>(signer: signer, receiver: address, amount: u128, period: u64, exec_delay: u64){
        TreasuryWithdrawDaoProposal::propose_withdraw<TokenT>(&signer, receiver, amount, period, exec_delay)
    }

    spec propose_withdraw {
        pragma verify = false;
    }

    public(script) fun execute_withdraw_proposal<TokenT:copy + drop + store>(signer: signer, proposer_address: address,
                                                                       proposal_id: u64,){
        TreasuryWithdrawDaoProposal::execute_withdraw_proposal<TokenT>(&signer, proposer_address, proposal_id);
    }

     spec execute_withdraw_proposal {
         pragma verify = false;
     }
}
}