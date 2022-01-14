address StarcoinFramework {
module ModuleUpgradeScripts {
    use StarcoinFramework::PackageTxnManager;
    use StarcoinFramework::Config;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Version;
    use StarcoinFramework::Option;
    use StarcoinFramework::UpgradeModuleDaoProposal;

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

    ///Update `sender`'s module upgrade strategy to `strategy`
    public(script) fun update_module_upgrade_strategy(
        sender: signer,
        strategy: u8,
    ) {
        // 1. check version
        if (strategy == PackageTxnManager::get_strategy_two_phase()) {
            if (!Config::config_exist_by_address<Version::Version>(Signer::address_of(&sender))) {
                Config::publish_new_config<Version::Version>(&sender, Version::new_version(1));
            }
        };

        // 2. update strategy
        PackageTxnManager::update_module_upgrade_strategy(
            &sender,
            strategy,
            Option::none<u64>(),
        );
    }

    /// a alias of execute_module_upgrade_plan_propose, will deprecated in the future.
    public(script) fun submit_module_upgrade_plan<Token: copy + drop + store>(
        sender: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        Self::execute_module_upgrade_plan_propose<Token>(sender, proposer_address, proposal_id);
    }

    ///Execute module upgrade plan propose by submit module upgrade plan, the propose must been agreed, and anyone can execute this function.
    public(script) fun execute_module_upgrade_plan_propose<Token: copy + drop + store>(
        _sender: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        UpgradeModuleDaoProposal::submit_module_upgrade_plan<Token>(proposer_address, proposal_id);
    }

    spec execute_module_upgrade_plan_propose {
        pragma verify = false;
    }

    ///Directly submit a upgrade plan, the `sender`'s module upgrade plan must been PackageTxnManager::STRATEGY_TWO_PHASE and have UpgradePlanCapability
    public(script) fun submit_upgrade_plan(sender: signer, package_hash: vector<u8>, version:u64, enforced: bool) {
        PackageTxnManager::submit_upgrade_plan_v2(&sender, package_hash, version, enforced);
    }

    spec submit_upgrade_plan {
        pragma verify = false;
    }

    ///Cancel current upgrade plan, the `sender` must have `UpgradePlanCapability`.
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