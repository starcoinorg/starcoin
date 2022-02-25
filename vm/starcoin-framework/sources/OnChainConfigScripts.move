address StarcoinFramework {
module OnChainConfigScripts {
    use StarcoinFramework::ConsensusConfig;
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::STC;
    use StarcoinFramework::RewardConfig;
    use StarcoinFramework::TransactionPublishOption;
    use StarcoinFramework::TransactionTimeoutConfig;
    use StarcoinFramework::VMConfig;
    use StarcoinFramework::Signer;
    use StarcoinFramework::LanguageVersion;

    public ( script ) fun propose_update_consensus_config(account: signer,
                                                          uncle_rate_target: u64,
                                                          base_block_time_target: u64,
                                                          base_reward_per_block: u128,
                                                          base_reward_per_uncle_percent: u64,
                                                          epoch_block_count: u64,
                                                          base_block_difficulty_window: u64,
                                                          min_block_time_target: u64,
                                                          max_block_time_target: u64,
                                                          base_max_uncles_per_block: u64,
                                                          base_block_gas_limit: u64,
                                                          strategy: u8,
                                                          exec_delay: u64) {
        let consensus_config = ConsensusConfig::new_consensus_config(uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            base_reward_per_uncle_percent,
            epoch_block_count,
            base_block_difficulty_window,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy);
        OnChainConfigDao::propose_update<STC::STC, ConsensusConfig::ConsensusConfig>(&account, consensus_config, exec_delay);
    }

    spec propose_update_consensus_config {
        pragma verify = false;
    }

    public ( script ) fun propose_update_reward_config(account: signer,
                                                       reward_delay: u64,
                                                       exec_delay: u64) {
        let reward_config = RewardConfig::new_reward_config(reward_delay);
        OnChainConfigDao::propose_update<STC::STC, RewardConfig::RewardConfig>(&account, reward_config, exec_delay);
    }

    spec propose_update_reward_config {
        pragma verify = false;
    }

    public ( script ) fun propose_update_txn_publish_option(account: signer,
                                                            script_allowed: bool,
                                                            module_publishing_allowed: bool,
                                                            exec_delay: u64) {
        let txn_publish_option = TransactionPublishOption::new_transaction_publish_option(script_allowed, module_publishing_allowed);
        OnChainConfigDao::propose_update<STC::STC, TransactionPublishOption::TransactionPublishOption>(&account, txn_publish_option, exec_delay);
    }

    spec propose_update_txn_publish_option {
        pragma verify = false;
    }

    public ( script ) fun propose_update_txn_timeout_config(account: signer,
                                                            duration_seconds: u64,
                                                            exec_delay: u64) {
        let txn_timeout_config = TransactionTimeoutConfig::new_transaction_timeout_config(duration_seconds);
        OnChainConfigDao::propose_update<STC::STC, TransactionTimeoutConfig::TransactionTimeoutConfig>(&account, txn_timeout_config, exec_delay);
    }

    spec propose_update_txn_timeout_config {
        pragma verify = false;
    }

    public ( script ) fun propose_update_vm_config(account: signer,
                                                   instruction_schedule: vector<u8>,
                                                   native_schedule: vector<u8>,
                                                   global_memory_per_byte_cost: u64,
                                                   global_memory_per_byte_write_cost: u64,
                                                   min_transaction_gas_units: u64,
                                                   large_transaction_cutoff: u64,
                                                   instrinsic_gas_per_byte: u64,
                                                   maximum_number_of_gas_units: u64,
                                                   min_price_per_gas_unit: u64,
                                                   max_price_per_gas_unit: u64,
                                                   max_transaction_size_in_bytes: u64,
                                                   gas_unit_scaling_factor: u64,
                                                   default_account_size: u64,
                                                   exec_delay: u64, ) {
        let vm_config = VMConfig::new_vm_config(instruction_schedule,
            native_schedule,
            global_memory_per_byte_cost,
            global_memory_per_byte_write_cost,
            min_transaction_gas_units,
            large_transaction_cutoff,
            instrinsic_gas_per_byte,
            maximum_number_of_gas_units,
            min_price_per_gas_unit,
            max_price_per_gas_unit,
            max_transaction_size_in_bytes,
            gas_unit_scaling_factor,
            default_account_size);
        OnChainConfigDao::propose_update<STC::STC, VMConfig::VMConfig>(&account, vm_config, exec_delay);
    }

    spec propose_update_vm_config {
        pragma verify = false;
    }

    public(script) fun propose_update_move_language_version(account: signer, new_version: u64, exec_delay: u64) {
        let lang_version = LanguageVersion::new(new_version);
        OnChainConfigDao::propose_update<STC::STC, LanguageVersion::LanguageVersion>(&account, lang_version, exec_delay);
    }

    spec propose_update_move_language_version {
        pragma verify = false;
    }

    public ( script ) fun execute_on_chain_config_proposal<ConfigT: copy + drop + store>(account: signer, proposal_id: u64) {
        OnChainConfigDao::execute<STC::STC, ConfigT>(Signer::address_of(&account), proposal_id);
    }

    spec execute_on_chain_config_proposal {
        pragma verify = false;
    }

    public(script) fun execute_on_chain_config_proposal_v2<TokenType: copy + drop + store, ConfigT: copy + drop + store>(proposer_address: address, proposal_id: u64) {
        OnChainConfigDao::execute<TokenType, ConfigT>(proposer_address, proposal_id);
    }

    spec execute_on_chain_config_proposal_v2 {
        pragma verify = false;
    }
}
}