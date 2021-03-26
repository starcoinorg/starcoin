address 0x1 {
module DaoVoteScripts {
    use 0x1::Dao;
    use 0x1::Account;
    use 0x1::Signer;

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
        let votes = Account::withdraw<Token>(&signer, votes);
        Dao::cast_vote<Token, ActionT>(&signer, proposer_address, proposal_id, votes, agree);
    }

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