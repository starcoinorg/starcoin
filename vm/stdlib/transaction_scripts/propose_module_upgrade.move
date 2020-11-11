script {
    use 0x1::UpgradeModuleDaoProposal;

    fun propose_module_upgrade<Token: copyable>(
        signer: &signer,
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
        exec_delay: u64,
    ) {
        UpgradeModuleDaoProposal::propose_module_upgrade<Token>(
            signer,
            module_address,
            package_hash,
            version,
            exec_delay,
        );
    }
}
