script {
    use 0x1::Dao;
    fun destroy_terminated_proposal<Token: copyable, Action: copyable>(
        _signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        Dao::destroy_terminated_proposal<Token, Action>(proposer_address, proposal_id);
    }
}
