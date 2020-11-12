script {
    use 0x1::Dao;
    use 0x1::Account;
    use 0x1::Signer;
    fun unstake_vote<Token: copyable, Action: copyable>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        let my_token = Dao::unstake_votes<Token, Action>(signer, proposer_address, proposal_id);
        Account::deposit(Signer::address_of(signer), my_token);
    }
}
