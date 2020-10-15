script {
    use 0x1::ModifyDaoConfigProposal;
    fun propose_modify_dao_config<Token: copyable>(
        signer: &signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
        exec_delay: u64,
    ) {
        ModifyDaoConfigProposal::propose<Token>(signer, voting_delay, voting_period, voting_quorum_rate, min_action_delay, exec_delay);
    }
}
