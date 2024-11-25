module starcoin_framework::on_chain_config_scripts {

    use std::signer;

    use starcoin_framework::block_reward_config;
    use starcoin_framework::consensus_config;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_language_version;
    use starcoin_framework::stc_transaction_timeout_config;
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::vm_config;

    public entry fun propose_update_consensus_config(
        account: signer,
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
        exec_delay: u64
    ) {
        let consensus_config = consensus_config::new_consensus_config(
            uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            base_reward_per_uncle_percent,
            epoch_block_count,
            base_block_difficulty_window,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy
        );

        on_chain_config_dao::propose_update<STC, consensus_config::ConsensusConfig>(
            &account,
            consensus_config,
            exec_delay
        );
    }

    spec propose_update_consensus_config {
        pragma verify = false;
    }

    public entry fun propose_update_reward_config(
        account: signer,
        reward_delay: u64,
        exec_delay: u64
    ) {
        let reward_config = block_reward_config::new_reward_config(reward_delay);
        on_chain_config_dao::propose_update<STC, block_reward_config::RewardConfig>(
            &account,
            reward_config,
            exec_delay
        );
    }

    spec propose_update_reward_config {
        pragma verify = false;
    }

    /// Propose to update the transaction publish option.
    ///
    public entry fun propose_update_txn_publish_option(
        account: signer,
        script_allowed: bool,
        module_publishing_allowed: bool,
        exec_delay: u64
    ) {
        let txn_publish_option = transaction_publish_option::new_transaction_publish_option(
            script_allowed,
            module_publishing_allowed
        );
        on_chain_config_dao::propose_update<STC, transaction_publish_option::TransactionPublishOption>(
            &account,
            txn_publish_option,
            exec_delay
        );
    }

    spec propose_update_txn_publish_option {
        pragma verify = false;
    }

    /// Propose to update the transaction timeout configuration.
    public entry fun propose_update_txn_timeout_config(
        account: signer,
        duration_seconds: u64,
        exec_delay: u64
    ) {
        let txn_timeout_config = stc_transaction_timeout_config::new_transaction_timeout_config(duration_seconds);
        on_chain_config_dao::propose_update<STC, stc_transaction_timeout_config::TransactionTimeoutConfig>(
            &account,
            txn_timeout_config,
            exec_delay
        );
    }

    spec propose_update_txn_timeout_config {
        pragma verify = false;
    }

    /// Propose to update the VM configuration.
    public entry fun propose_update_vm_config(account: signer, new_config: vector<u8>, exec_delay: u64) {
        let new_config = vm_config::new_from_blob(new_config);
        on_chain_config_dao::propose_update<STC, vm_config::VMConfig>(&account, new_config, exec_delay);
    }

    spec propose_update_vm_config {
        pragma verify = false;
    }

    public entry fun propose_update_move_language_version(account: signer, new_version: u64, exec_delay: u64) {
        let lang_version = stc_language_version::new(new_version);
        on_chain_config_dao::propose_update<STC, stc_language_version::LanguageVersion>(
            &account,
            lang_version,
            exec_delay
        );
    }

    spec propose_update_move_language_version {
        pragma verify = false;
    }

    // TODO(BobOng): [framework compatible] To implement the following functions, we need to implement the `FlexiDagConfig` struct.
    // public entry fun propose_update_flexi_dag_effective_height(account: signer, new_height: u64, exec_delay: u64) {
    //     let config = FlexiDagConfig::new_flexidag_config(new_height);
    //     OnChainConfigDao::propose_update<STC::STC, FlexiDagConfig::FlexiDagConfig>(&account, config, exec_delay);
    // }

    // spec propose_update_flexi_dag_effective_height {
    //     pragma verify = false;
    // }

    public entry fun execute_on_chain_config_proposal<ConfigT: copy + drop + store>(account: signer, proposal_id: u64) {
        on_chain_config_dao::execute<STC, ConfigT>(signer::address_of(&account), proposal_id);
    }

    spec execute_on_chain_config_proposal {
        pragma verify = false;
    }

    public entry fun execute_on_chain_config_proposal_v2<TokenType, ConfigT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64
    ) {
        on_chain_config_dao::execute<TokenType, ConfigT>(proposer_address, proposal_id);
    }

    spec execute_on_chain_config_proposal_v2 {
        pragma verify = false;
    }
}