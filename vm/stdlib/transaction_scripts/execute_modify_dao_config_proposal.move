script {
    use 0x1::ModifyDaoConfigProposal;

    fun execute_modify_dao_config_proposal<Token: copyable>(
        _signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        ModifyDaoConfigProposal::execute<Token>(proposer_address, proposal_id);
    }
}
