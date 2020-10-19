script {
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun revoke_vote<Token: copyable, Action>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let sender = Signer::address_of(signer);
        let (_, power) = Dao::vote_of<Token>(sender, proposer_address, proposal_id);
        let my_token = Dao::revoke_vote<Token, Action>(signer, proposer_address, proposal_id, power);
        Account::deposit(sender, my_token);
    }
}
