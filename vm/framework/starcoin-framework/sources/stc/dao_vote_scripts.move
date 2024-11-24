module starcoin_framework::dao_vote_scripts {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::dao;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = true;
    }

    public entry fun cast_vote<Token, ActionT: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let sender = signer::address_of(&signer);
        if (dao::has_vote<Token>(sender, proposer_address, proposal_id)) {
            // if already voted, and vote is not same as the current cast, change the existing vote.
            // resolve https://github.com/starcoinorg/starcoin/issues/2925.
            let (agree_voted, _) = dao::vote_of<Token>(sender, proposer_address, proposal_id);
            if (agree_voted != agree) {
                dao::change_vote<Token, ActionT>(&signer, proposer_address, proposal_id, agree);
            }
        };

        let votes = coin::withdraw<Token>(&signer, (votes as u64));
        dao::cast_vote<Token, ActionT>(&signer, proposer_address, proposal_id, votes, agree);
    }

    /// revoke all votes on a proposal
    public entry fun revoke_vote<Token, Action: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let sender = signer::address_of(&signer);
        let (_, power) = dao::vote_of<Token>(sender, proposer_address, proposal_id);
        let my_token = dao::revoke_vote<Token, Action>(&signer, proposer_address, proposal_id, power);
        coin::deposit(sender, my_token);
    }

    /// Let user change their vote during the voting time.
    public entry fun flip_vote<TokenT, ActionT: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let (agree, _) = dao::vote_of<TokenT>(signer::address_of(&signer), proposer_address, proposal_id);
        dao::change_vote<TokenT, ActionT>(&signer, proposer_address, proposal_id, !agree);
    }

    /// revoke some votes on a proposal
    public entry fun revoke_vote_of_power<Token, Action: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        power: u128,
    ) {
        let sender = signer::address_of(&signer);
        let my_token = dao::revoke_vote<Token, Action>(&signer, proposer_address, proposal_id, power);
        coin::deposit(sender, my_token);
    }

    public entry fun unstake_vote<Token, Action: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let my_token = dao::unstake_votes<Token, Action>(&signer, proposer_address, proposal_id);
        coin::deposit(signer::address_of(&signer), my_token);
    }
}