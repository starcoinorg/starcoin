script {
    use 0x1::PackageTxnManager;
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::Version;

    fun update_module_upgrade_strategy(
        signer: &signer,
        strategy: u8,
    ) {
        // 1. check version
        if (strategy == PackageTxnManager::get_strategy_two_phase()){
            if (!Config::config_exist_by_address<Version::Version>(Signer::address_of(signer))) {
                Config::publish_new_config<Version::Version>(signer, Version::new_version(1));
            }
        };

        // 2. update strategy
        PackageTxnManager::update_module_upgrade_strategy(
            signer,
            strategy,
        );
    }
}
