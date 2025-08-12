module starcoin_framework::treasury_scripts {

    use std::signer;

    use starcoin_framework::coin;
    use starcoin_framework::dao_treasury_withdraw_proposal;
    use starcoin_framework::stc_offer;
    use starcoin_framework::treasury;

    /// Withdraw token from treasury and split the LinearWithdrawCapability.
    public entry fun withdraw_and_split_lt_withdraw_cap<TokenT>(
        account: signer,
        for_address: address,
        amount: u128,
        lock_period: u64,
    ) {
        // 1. take cap: LinearWithdrawCapability<TokenT>
        let cap = treasury::remove_linear_withdraw_capability<TokenT>(&account);

        // 2. withdraw token and split
        let (tokens, new_cap) = treasury::split_linear_withdraw_cap(&mut cap, amount);

        // 3. deposit
        coin::deposit(signer::address_of(&account), tokens);

        // 4. put or destroy key
        if (treasury::is_empty_linear_withdraw_capability(&cap)) {
            treasury::destroy_linear_withdraw_capability(cap);
        } else {
            treasury::add_linear_withdraw_capability(&account, cap);
        };

        // 5. offer
        stc_offer::create(&account, new_cap, for_address, lock_period);
    }

    spec withdraw_and_split_lt_withdraw_cap {
        pragma verify = false;
    }

    /// Withdraw token from treasury.
    public entry fun withdraw_token_with_linear_withdraw_capability<TokenT>(account: signer) {
        // 1. take cap
        let cap = treasury::remove_linear_withdraw_capability<TokenT>(&account);

        // 2. withdraw token
        let tokens = treasury::withdraw_with_linear_capability(&mut cap);

        // 3. deposit
        coin::deposit(signer::address_of(&account), tokens);

        // 4. put or destroy key
        if (treasury::is_empty_linear_withdraw_capability(&cap)) {
            treasury::destroy_linear_withdraw_capability(cap);
        } else {
            treasury::add_linear_withdraw_capability(&account, cap);
        };
    }

    spec withdraw_token_with_linear_withdraw_capability {
        pragma verify = false;
    }

    /// Propose a withdraw from treasury.
    public entry fun propose_withdraw<TokenT>(
        signer: signer,
        receiver: address,
        amount: u128,
        period: u64,
        exec_delay: u64
    ) {
        dao_treasury_withdraw_proposal::propose_withdraw<TokenT>(&signer, receiver, amount, period, exec_delay)
    }

    spec propose_withdraw {
        pragma verify = false;
    }

    /// Execute a withdraw proposal.
    public entry fun execute_withdraw_proposal<TokenT>(signer: signer, proposer_address: address, proposal_id: u64) {
        dao_treasury_withdraw_proposal::execute_withdraw_proposal<TokenT>(&signer, proposer_address, proposal_id);
    }

    spec execute_withdraw_proposal {
        pragma verify = false;
    }
}