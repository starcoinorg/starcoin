script {
    use 0x1::Dao;
    use 0x1::Account;
    fun unstake_vote<Token: copyable, Action>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let my_token = Dao::unstake_votes<Token, Action>(signer, proposer_address, proposal_id);
        Account::deposit(signer, my_token);
    }
}
