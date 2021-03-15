script {
    use 0x1::UpgradeModuleDaoProposal;

    fun submit_module_upgrade_plan<Token: copy + drop + store>(
        _signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        UpgradeModuleDaoProposal::submit_module_upgrade_plan<Token>(proposer_address, proposal_id);
    }
}
