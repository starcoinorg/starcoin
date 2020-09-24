script {
    use 0x1::Dao;
    use 0x1::Account;

    fun cast_vote<Token: copyable, ActionT>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let votes = Account::withdraw<Token>(signer, votes);
        Dao::cast_vote<Token, ActionT>(signer, proposer_address, proposal_id, votes, agree);
    }
}