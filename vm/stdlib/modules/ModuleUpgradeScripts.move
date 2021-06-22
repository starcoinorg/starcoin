address 0x1 {
module ModuleUpgradeScripts {
    use 0x1::PackageTxnManager;
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::Version;
    use 0x1::Option;
    use 0x1::UpgradeModuleDaoProposal;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = true;
    }

    public(script) fun propose_module_upgrade_v2<Token: copy + drop + store>(
        signer: signer,
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
        exec_delay: u64,
        enforced: bool,
    ) {
        UpgradeModuleDaoProposal::propose_module_upgrade_v2<Token>(
            &signer,
            module_address,
            package_hash,
            version,
            exec_delay,
            enforced
        );
    }
    public(script) fun update_module_upgrade_strategy(
        signer: signer,
        strategy: u8,
    ) {
        // 1. check version
        if (strategy == PackageTxnManager::get_strategy_two_phase()) {
            if (!Config::config_exist_by_address<Version::Version>(Signer::address_of(&signer))) {
                Config::publish_new_config<Version::Version>(&signer, Version::new_version(1));
            }
        };

        // 2. update strategy
        PackageTxnManager::update_module_upgrade_strategy(
            &signer,
            strategy,
            Option::none<u64>(),
        );
    }


    public(script) fun submit_module_upgrade_plan<Token: copy + drop + store>(
        _signer: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        UpgradeModuleDaoProposal::submit_module_upgrade_plan<Token>(proposer_address, proposal_id);
    }

    public(script) fun cancel_upgrade_plan(
        signer: signer,
    ) {
        PackageTxnManager::cancel_upgrade_plan(&signer);
    }

    spec cancel_upgrade_plan {
        pragma verify = false;
    }
}
}