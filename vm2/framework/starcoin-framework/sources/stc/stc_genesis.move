/// The module for init Genesis
module starcoin_framework::stc_genesis {

    use std::error;
    use std::features;
    use std::option;
    use std::vector;

    use starcoin_framework::account;
    use starcoin_framework::aggregator_factory;
    use starcoin_framework::block_reward;
    use starcoin_framework::block_reward_config;
    use starcoin_framework::chain_id;
    use starcoin_framework::coin;
    use starcoin_framework::consensus_config;
    use starcoin_framework::consensus_strategy;
    use starcoin_framework::dao;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::dao_treasury_withdraw_proposal;
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::epoch;
    use starcoin_framework::fungible_asset::{Self, FungibleAsset};
    use starcoin_framework::on_chain_config;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::oracle_stc_usd;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_block;
    use starcoin_framework::stc_language_version;
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::stc_transaction_timeout_config;
    use starcoin_framework::stc_util;
    use starcoin_framework::stc_version;
    use starcoin_framework::system_addresses;
    use starcoin_framework::timestamp;
    use starcoin_framework::transaction_fee;
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::treasury;
    use starcoin_framework::vm_config;
    use starcoin_std::debug;

    const ERR_INITIALIZE_STC_AMOUNT_ERROR: u64 = 1;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = true;
    }

    public entry fun initialize(
        stdlib_version: u64,
        // block reward and stc config
        reward_delay: u64,
        total_stc_amount: u128,
        pre_mine_stc_amount: u128,
        time_mint_stc_amount: u128,
        time_mint_stc_period: u64,
        parent_hash: vector<u8>,
        association_auth_key: vector<u8>,
        genesis_auth_key: vector<u8>,
        chain_id: u8,
        genesis_timestamp: u64,
        //consensus config
        uncle_rate_target: u64,
        epoch_block_count: u64,
        base_block_time_target: u64,
        base_block_difficulty_window: u64,
        base_reward_per_block: u128,
        base_reward_per_uncle_percent: u64,
        min_block_time_target: u64,
        max_block_time_target: u64,
        base_max_uncles_per_block: u64,
        base_block_gas_limit: u64,
        strategy: u8,
        max_transaction_per_block: u64,
        pruning_depth: u64,
        pruning_finality: u64,
        //vm config
        script_allowed: bool,
        module_publishing_allowed: bool,
        gas_schedule_blob: vector<u8>,
        // dao config
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
        // transaction timeout config
        transaction_timeout: u64,
        features: vector<u8>,
    ) {
        debug::print(&std::string::utf8(b"stc_genesis::initialize Entered"));


        // create genesis account
        let (starcoin_framework_account, _genesis_signer_cap) =
            account::create_framework_reserved_account(@starcoin_framework);

        // Initialize features
        features::initialize(
            &starcoin_framework_account,
            features
        );

        // Initialize versions
        initialize_versions(&starcoin_framework_account, stdlib_version);

        // Initalize aggreator factorys
        aggregator_factory::initialize_aggregator_factory(&starcoin_framework_account);

        // Init global time
        timestamp::set_time_has_started(&starcoin_framework_account, genesis_timestamp * 1000);

        chain_id::initialize(&starcoin_framework_account, chain_id);

        consensus_strategy::initialize(&starcoin_framework_account, strategy);

        stc_block::initialize(&starcoin_framework_account, parent_hash);

        transaction_publish_option::initialize(
            &starcoin_framework_account,
            script_allowed,
            module_publishing_allowed,
        );

        // init config
        vm_config::initialize(
            &starcoin_framework_account,
            gas_schedule_blob,
        );

        stc_transaction_timeout_config::initialize(&starcoin_framework_account, transaction_timeout);
        consensus_config::initialize(
            &starcoin_framework_account,
            uncle_rate_target,
            epoch_block_count,
            base_block_time_target,
            base_block_difficulty_window,
            base_reward_per_block,
            base_reward_per_uncle_percent,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy,
            max_transaction_per_block,
            pruning_depth,
            pruning_finality,
        );

        epoch::initialize(&starcoin_framework_account);

        // stdlib use two phase upgrade strategy.
        stc_transaction_package_validation::update_module_upgrade_strategy(
            &starcoin_framework_account,
            stc_transaction_package_validation::get_strategy_two_phase(),
            option::some(0u64),
        );

        // Initliaze STC registration
        let total_supply_coin = Self::initialize_stc(
            &starcoin_framework_account,
            total_stc_amount
        );

        block_reward::initialize(&starcoin_framework_account, reward_delay);

        // Init DAO configuration
        Self::initialize_dao(
            &starcoin_framework_account,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );

        // Init goverances account
        let core_resource_account = account::create_account(@core_resources);
        coin::register<STC>(&core_resource_account);
        Self::initialize_stc_governance_allocation(
            &starcoin_framework_account,
            &core_resource_account,
            total_supply_coin,
            pre_mine_stc_amount,
            time_mint_stc_amount,
            time_mint_stc_period,
        );

        transaction_fee::initialize(&starcoin_framework_account);

        // Only test/dev network set genesis auth key.
        if (!vector::is_empty(&genesis_auth_key) && (stc_util::is_net_dev() || stc_util::is_net_test())) {
            account::rotate_authentication_key_internal(&starcoin_framework_account, genesis_auth_key);
        };
        account::rotate_authentication_key_internal(&core_resource_account, association_auth_key);

        // let assoc_rotate_key_cap = Account::extract_key_rotation_capability(&core_resource_account);
        // Account::rotate_authentication_key_with_capability(&assoc_rotate_key_cap, association_auth_key);
        // Account::restore_key_rotation_capability(assoc_rotate_key_cap);
        //
        // // v5 -> v6
        // {
        //     let cap = Account::remove_signer_capability(&genesis_account);
        //     GenesisSignerCapability::initialize(&genesis_account, cap);
        //     //register oracle
        //     STCUSDOracle::register(&genesis_account);
        //     let merkle_root = x"5969f0e8e19f8769276fb638e6060d5c02e40088f5fde70a6778dd69d659ee6d";
        //     let image = b"ipfs://QmSPcvcXgdtHHiVTAAarzTeubk5X3iWymPAoKBfiRFjPMY";
        //     GenesisNFT::initialize(&genesis_account, merkle_root, 1639u64, image);
        // };
        // StdlibUpgradeScripts::do_upgrade_from_v6_to_v7_with_language_version(&genesis_account, 6);
        // StdlibUpgradeScripts::do_upgrade_from_v11_to_v12(&genesis_account);

        // //Start time, Timestamp::is_genesis() will return false. this call should at the end of genesis init.
        // timestamp::set_time_has_started(&starcoin_framework_account);
        // account::release_genesis_signer(genesis_account);
        // account::release_genesis_signer(association);

        // Register oracle
        oracle_stc_usd::register(&starcoin_framework_account);

        debug::print(&std::string::utf8(b"stc_genesis::initialize | Exited"));
    }

    fun initialize_versions(starcoin_framework_account: &signer, stdlib_version: u64) {
        // Version initialization
        on_chain_config::publish_new_config<stc_version::Version>(
            starcoin_framework_account,
            stc_version::new_version(stdlib_version)
        );
        on_chain_config::publish_new_config<stc_language_version::LanguageVersion>(
            starcoin_framework_account,
            stc_language_version::new(13),
        );
    }

    /// First we need to initialize the STC token.
    /// Then we can initialize the treasury.
    /// The treasury will mint the total_stc_amount to the treasury.
    fun initialize_stc(
        starcoin_framework: &signer,
        total_stc_amount: u128,
    ): fungible_asset::FungibleAsset {
        debug::print(&std::string::utf8(b"stc_genesis::initialize_stc | Entered"));

        let (burn_cap, mint_cap) = starcoin_coin::initialize(starcoin_framework);
        coin::register<STC>(starcoin_framework);
        coin::create_coin_conversion_map(starcoin_framework);
        coin::create_pairing<STC>(starcoin_framework);
        transaction_fee::store_fee_caps(starcoin_framework, burn_cap, mint_cap);

        let (mint_ref, mint_ref_receipt) = coin::get_paired_mint_ref(&mint_cap);
        let fa_stc = fungible_asset::mint(&mint_ref, (total_stc_amount as u64));
        coin::return_paired_mint_ref(mint_ref, mint_ref_receipt);

        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);

        assert!(
            (fungible_asset::amount(&fa_stc) as u128) == total_stc_amount,
            error::invalid_state(ERR_INITIALIZE_STC_AMOUNT_ERROR)
        );
        debug::print(&std::string::utf8(b"stc_genesis::initialize_stc | Exited"));
        fa_stc
    }

    fun initialize_dao(
        starcoin_framework: &signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64
    ) {
        debug::print(&std::string::utf8(b"stc_genesis::initialize_dao | Entered "));

        dao::plugin<STC>(
            starcoin_framework,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );
        dao_modify_config_proposal::plugin<STC>(starcoin_framework);


        let upgrade_plan_cap = stc_transaction_package_validation::extract_submit_upgrade_plan_cap(starcoin_framework);
        dao_upgrade_module_proposal::plugin<STC>(starcoin_framework, upgrade_plan_cap);

        // the following configurations are gov-ed by Dao.
        on_chain_config_dao::plugin<STC, transaction_publish_option::TransactionPublishOption>(starcoin_framework);
        on_chain_config_dao::plugin<STC, vm_config::VMConfig>(starcoin_framework);
        on_chain_config_dao::plugin<STC, consensus_config::ConsensusConfig>(starcoin_framework);
        on_chain_config_dao::plugin<STC, block_reward_config::RewardConfig>(starcoin_framework);
        on_chain_config_dao::plugin<STC, stc_transaction_timeout_config::TransactionTimeoutConfig>(starcoin_framework);

        debug::print(&std::string::utf8(b"stc_genesis::initialize_dao | Exited"));
    }

    /// Overall governance allocation strategy:
    /// 1. `pre_mine_stc_amount` of the total supply is allocated to the Association.
    /// 2. `time_mint_stc_amount` of the total supply is allocated to the Association linearly over `time_mint_stc_period` blocks.
    fun initialize_stc_governance_allocation(
        starcoin_framework: &signer,
        core_resource_account: &signer,
        total_supply_stc: FungibleAsset,
        pre_mine_stc_amount: u128,
        time_mint_stc_amount: u128,
        time_mint_stc_period: u64,
    ) {
        debug::print(&std::string::utf8(b"stc_genesis::initialize_stc_governance_allocation | Entered"));

        let treasury_withdraw_cap = treasury::initialize<STC>(starcoin_framework, total_supply_stc);

        if (pre_mine_stc_amount > 0) {
            let core_resource_address = system_addresses::get_core_resource_address();
            let stc = treasury::withdraw_with_capability<STC>(
                &mut treasury_withdraw_cap,
                pre_mine_stc_amount
            );
            primary_fungible_store::deposit(core_resource_address, stc);
        };
        if (time_mint_stc_amount > 0) {
            let liner_withdraw_cap = treasury::issue_linear_withdraw_capability<STC>(
                &mut treasury_withdraw_cap,
                time_mint_stc_amount,
                time_mint_stc_period
            );
            treasury::add_linear_withdraw_capability(core_resource_account, liner_withdraw_cap);
        };
        dao_treasury_withdraw_proposal::plugin<STC>(starcoin_framework, treasury_withdraw_cap);

        debug::print(&std::string::utf8(b"stc_genesis::initialize_stc_governance_allocation | Exited"));
    }

    #[test_only]
    public fun initialize_for_unit_tests() {
        let stdlib_version: u64 = 6;
        let reward_delay: u64 = 7;
        let total_stc_amount: u128 = 3185136000000000000u128;
        let pre_mine_stc_amount: u128 = 159256800000000000u128;
        let time_mint_stc_amount: u128 = (85043130u128 * 3u128 + 74213670u128 * 3u128) * 1000000000u128;
        let time_mint_stc_period: u64 = 1000000000;

        let parent_hash: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
        let association_auth_key: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
        let genesis_auth_key: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
        let chain_id: u8 = 255;
        let genesis_timestamp: u64 = 0;

        //consensus config
        let uncle_rate_target: u64 = 240;  // DAG mode: expect few uncles
        let epoch_block_count: u64 = 240;
        let base_block_time_target: u64 = 10000;
        let base_block_difficulty_window: u64 = 24;
        let base_reward_per_block: u128 = 1000000000;
        let base_reward_per_uncle_percent: u64 = 10;
        let min_block_time_target: u64 = 5000;
        let max_block_time_target: u64 = 60000;
        let base_max_uncles_per_block: u64 = 2;
        let base_block_gas_limit: u64 = 500000000;
        let strategy: u8 = 0;
        let max_transaction_per_block: u64 = 3000;  // DAG: limit transactions per block
        let pruning_depth: u64 = 185798;  // DAG pruning parameters
        let pruning_finality: u64 = 86400;

        //vm config
        let script_allowed: bool = true;
        let module_publishing_allowed: bool = true;

        // todo: initialize gas_schedule_blob properly
        let gas_schedule_blob: vector<u8> = vector::empty<u8>();

        // dao config
        let voting_delay: u64 = 1000;
        let voting_period: u64 = 6000;
        let voting_quorum_rate: u8 = 4;
        let min_action_delay: u64 = 1000;

        // transaction timeout config
        let transaction_timeout: u64 = 10000;

        Self::initialize(
            stdlib_version,
            reward_delay,
            total_stc_amount,
            pre_mine_stc_amount,
            time_mint_stc_amount,
            time_mint_stc_period,
            parent_hash,
            association_auth_key,
            genesis_auth_key,
            chain_id,
            genesis_timestamp,
            uncle_rate_target,
            epoch_block_count,
            base_block_time_target,
            base_block_difficulty_window,
            base_reward_per_block,
            base_reward_per_uncle_percent,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy,
            max_transaction_per_block,
            pruning_depth,
            pruning_finality,
            script_allowed,
            module_publishing_allowed,
            gas_schedule_blob,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
            transaction_timeout,
            vector::empty(),
        );
    }

    #[test(framework = @0x1, root = @0xabcd)]
    fun test_create_root_account(framework: &signer) {
        use starcoin_framework::fungible_asset::Metadata;
        use starcoin_framework::primary_fungible_store;
        use starcoin_framework::object;
        use starcoin_framework::starcoin_account;
        use std::features;

        let feature = features::get_new_accounts_default_to_fa_stc_store_feature();
        features::change_feature_flags_for_testing(framework, vector[feature], vector[]);

        aggregator_factory::initialize_aggregator_factory_for_test(framework);

        let (burn_cap, mint_cap) = starcoin_coin::initialize(framework);
        starcoin_coin::ensure_initialized_with_stc_fa_metadata_for_test();

        let core_resources = account::create_account(@core_resources);
        starcoin_account::register_stc(&core_resources); // registers STC store

        let apt_metadata = object::address_to_object<Metadata>(@starcoin_fungible_asset);
        assert!(primary_fungible_store::primary_store_exists(@core_resources, apt_metadata), 2);

        starcoin_coin::configure_accounts_for_test(framework, &core_resources, mint_cap);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }
}
