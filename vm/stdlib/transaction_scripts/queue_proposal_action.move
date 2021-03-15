script {
    use 0x1::Dao;

    fun queue_proposal_action<Token: copy + drop + store, Action: copy + drop + store>(
        _signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        Dao::queue_proposal_action<Token, Action>(proposer_address, proposal_id);
    }
}
