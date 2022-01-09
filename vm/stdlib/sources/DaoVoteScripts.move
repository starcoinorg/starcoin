address StarcoinFramework {
module DaoVoteScripts {
    use StarcoinFramework::Dao;
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = true;
    }

    public ( script ) fun cast_vote<Token: copy + drop + store, ActionT: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let sender = Signer::address_of(&signer);
        if (Dao::has_vote<Token>(sender, proposer_address, proposal_id)) {
            // if already voted, and vote is not same as the current cast, change the existing vote.
            // resolve https://github.com/starcoinorg/starcoin/issues/2925.
            let (agree_voted, _) = Dao::vote_of<Token>(sender, proposer_address, proposal_id);
            if (agree_voted != agree) {
                Dao::change_vote<Token, ActionT>(&signer, proposer_address, proposal_id, agree);
            }
        };

        let votes = Account::withdraw<Token>(&signer, votes);
        Dao::cast_vote<Token, ActionT>(&signer, proposer_address, proposal_id, votes, agree);
    }

    /// revoke all votes on a proposal
    public ( script ) fun revoke_vote<Token: copy + drop + store, Action: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let sender = Signer::address_of(&signer);
        let (_, power) = Dao::vote_of<Token>(sender, proposer_address, proposal_id);
        let my_token = Dao::revoke_vote<Token, Action>(&signer, proposer_address, proposal_id, power);
        Account::deposit(sender, my_token);
    }

    /// Let user change their vote during the voting time.
    public(script) fun flip_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let (agree, _) = Dao::vote_of<TokenT>(Signer::address_of(&signer), proposer_address, proposal_id);
        Dao::change_vote<TokenT, ActionT>(&signer, proposer_address, proposal_id, !agree);
    }

    /// revoke some votes on a proposal
    public ( script ) fun revoke_vote_of_power<Token: copy + drop + store, Action: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        power: u128,
    ) {
        let sender = Signer::address_of(&signer);
        let my_token = Dao::revoke_vote<Token, Action>(&signer, proposer_address, proposal_id, power);
        Account::deposit(sender, my_token);
    }

    public ( script ) fun unstake_vote<Token: copy + drop + store, Action: copy + drop + store>(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let my_token = Dao::unstake_votes<Token, Action>(&signer, proposer_address, proposal_id);
        Account::deposit(Signer::address_of(&signer), my_token);
    }
}
}