script {
    use 0x1::CoreAddresses;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::TransactionTimeout;
    use 0x1::Timestamp;
    use 0x1::Token;
    use 0x1::STC::{Self, STC};
    use 0x1::DummyToken;
    use 0x1::PackageTxnManager;
    use 0x1::ConsensusConfig;
    use 0x1::Version;
    use 0x1::VMConfig;
    use 0x1::Vector;
    use 0x1::Block;
    use 0x1::TransactionFee;
    use 0x1::BlockReward;
    use 0x1::ChainId;
    use 0x1::ConsensusStrategy;
    use 0x1::TransactionPublishOption;
    use 0x1::TokenLockPool;
    use 0x1::Box;

    fun genesis_init(

        reward_delay: u64,

        pre_mine_amount: u128,
        time_mint_amount: u128,
        time_mint_period: u64,
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

        //vm config
        merged_script_allow_list: vector<u8>,
        is_open_module: bool,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,

        //gas constants
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

        // dao config
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ) {
        assert(Timestamp::is_genesis(), 1);
        // create genesis account
        let genesis_account = Account::create_genesis_account(CoreAddresses::GENESIS_ADDRESS());
        //Init global time
        Timestamp::initialize(&genesis_account, genesis_timestamp);
        ChainId::initialize(&genesis_account, chain_id);
        ConsensusStrategy::initialize(&genesis_account, strategy);
        Block::initialize(&genesis_account, parent_hash);
        TransactionPublishOption::initialize(
            &genesis_account,
            merged_script_allow_list,
            is_open_module,
        );
        // init config
        VMConfig::initialize(
            &genesis_account,
            instruction_schedule,
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
            default_account_size,
        );
        Version::initialize(&genesis_account);
        TransactionTimeout::initialize(&genesis_account);
        ConsensusConfig::initialize(
            &genesis_account,
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
        );
        BlockReward::initialize(&genesis_account, reward_delay);
        TransactionFee::initialize(&genesis_account);
        let association = Account::create_genesis_account(
            CoreAddresses::ASSOCIATION_ROOT_ADDRESS(),
        );
        //Grant stdlib maintainer to association
        PackageTxnManager::grant_maintainer(&genesis_account, Signer::address_of(&association));
        // stdlib use two phase upgrade strategy.
        PackageTxnManager::update_module_upgrade_strategy(
            &genesis_account,
            PackageTxnManager::STRATEGY_TWO_PHASE(),
        );
        // stc should be initialized after genesis_account's module upgrade strategy set.
        {
            STC::initialize(&genesis_account, voting_delay, voting_period, voting_quorum_rate, min_action_delay);
            Account::accept_token<STC>(&genesis_account);
            DummyToken::initialize(&genesis_account);
            Account::accept_token<STC>(&association);
        };
        TokenLockPool::initialize(&genesis_account);
        if (pre_mine_amount > 0) {
            let stc = Token::mint<STC>(&genesis_account, pre_mine_amount);
            Account::deposit_to(&genesis_account, Signer::address_of(&association), stc);
        };
        if (time_mint_amount > 0) {
            let cap = Token::remove_mint_capability<STC>(&genesis_account);
            let key = Token::issue_linear_mint_key<STC>(&cap, time_mint_amount, time_mint_period);
            Token::add_mint_capability(&genesis_account, cap);
            Box::put(&association, key);
        };
        // only dev network set genesis auth key.
        if (!Vector::is_empty(&genesis_auth_key)) {
            let genesis_rotate_key_cap = Account::extract_key_rotation_capability(&genesis_account);
            Account::rotate_authentication_key(&genesis_rotate_key_cap, genesis_auth_key);
            Account::restore_key_rotation_capability(genesis_rotate_key_cap);
        };
        let assoc_rotate_key_cap = Account::extract_key_rotation_capability(&association);
        Account::rotate_authentication_key(&assoc_rotate_key_cap, association_auth_key);
        Account::restore_key_rotation_capability(assoc_rotate_key_cap);
        //Start time, Timestamp::is_genesis() will return false. this call should at the end of genesis init.
        Timestamp::set_time_has_started(&genesis_account);
        Account::release_genesis_signer(genesis_account);
        Account::release_genesis_signer(association);
    }
}