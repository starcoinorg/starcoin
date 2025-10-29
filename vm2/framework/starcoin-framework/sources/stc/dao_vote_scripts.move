module starcoin_framework::dao_vote_scripts {
    use std::error;
    use std::option;
    use std::signer;
    use starcoin_framework::fungible_asset;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::coin;
    use starcoin_framework::dao;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = true;
    }

    const ERR_CAST_VOTE_FA_NOT_FOUND: u64 = 1;
    const ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH: u64 = 2;

    public entry fun cast_vote<Token, ActionT: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let sender = signer::address_of(signer);
        if (dao::has_vote<Token>(sender, proposer_address, proposal_id)) {
            // if already voted, and vote is not same as the current cast, change the existing vote.
            // resolve https://github.com/starcoinorg/starcoin/issues/2925.
            let (agree_voted, _) = dao::vote_of<Token>(sender, proposer_address, proposal_id);
            if (agree_voted != agree) {
                dao::change_vote<Token, ActionT>(signer, proposer_address, proposal_id, agree);
            }
        };

        let metadata = coin::paired_metadata<Token>();
        assert!(option::is_some(&metadata), error::invalid_state(ERR_CAST_VOTE_FA_NOT_FOUND));

        let votes_fa = primary_fungible_store::withdraw(signer, option::destroy_some(metadata), (votes as u64));
        assert!(
            fungible_asset::amount(&votes_fa) == (votes as u64),
            error::invalid_state(ERR_CAST_VOTE_WITHDRAW_FA_NOT_MATCH)
        );

        dao::cast_vote<Token, ActionT>(signer, proposer_address, proposal_id, votes_fa, agree);
    }

    /// revoke all votes on a proposal
    public entry fun revoke_vote<Token, Action: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let sender = signer::address_of(signer);
        let (_, power) = dao::vote_of<Token>(sender, proposer_address, proposal_id);
        let my_token = dao::revoke_vote<Token, Action>(signer, proposer_address, proposal_id, power);
        primary_fungible_store::deposit(sender, my_token);
    }

    /// Let user change their vote during the voting time.
    public entry fun flip_vote<TokenT, ActionT: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let (agree, _) = dao::vote_of<TokenT>(signer::address_of(signer), proposer_address, proposal_id);
        dao::change_vote<TokenT, ActionT>(signer, proposer_address, proposal_id, !agree);
    }

    /// revoke some votes on a proposal
    public entry fun revoke_vote_of_power<Token, Action: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        power: u128,
    ) {
        let sender = signer::address_of(signer);
        let my_token = dao::revoke_vote<Token, Action>(signer, proposer_address, proposal_id, power);
        primary_fungible_store::deposit(sender, my_token);
    }

    public entry fun unstake_vote<Token, Action: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let my_token = dao::unstake_votes<Token, Action>(signer, proposer_address, proposal_id);
        primary_fungible_store::deposit(signer::address_of(signer), my_token);
    }
}