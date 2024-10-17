/// The module provide configuration of consensus parameters.
module starcoin_framework::consensus_config {

    use std::error;
    use std::signer;
    use starcoin_std::math128;
    use starcoin_framework::on_chain_config;
    use starcoin_framework::system_addresses;

    // friend starcoin_framework::stc_genesis;

    /// consensus configurations.
    struct ConsensusConfig has copy, drop, store {
        /// Uncle block rate per epoch
        uncle_rate_target: u64,
        /// Average target time to calculate a block's difficulty
        base_block_time_target: u64,
        /// Rewards per block
        base_reward_per_block: u128,
        /// Percentage of `base_reward_per_block` to reward a uncle block
        base_reward_per_uncle_percent: u64,
        /// Blocks each epoch
        epoch_block_count: u64,
        /// How many ancestor blocks which use to calculate next block's difficulty
        base_block_difficulty_window: u64,
        /// Minimum target time to calculate a block's difficulty
        min_block_time_target: u64,
        /// Maximum target time to calculate a block's difficulty
        max_block_time_target: u64,
        /// Maximum number of uncle block per block
        base_max_uncles_per_block: u64,
        /// Maximum gases per block
        base_block_gas_limit: u64,
        /// Strategy to calculate difficulty
        strategy: u8,
    }

    const EINVALID_ARGUMENT: u64 = 18;

    /// Initialization of the module.
    public(friend) fun initialize(
        account: &signer,
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
    ) {
        system_addresses::assert_starcoin_framework(account);

        on_chain_config::publish_new_config<Self::ConsensusConfig>(
            account,
            new_consensus_config(
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
                strategy,
            ),
        );
    }


    /// Create a new consensus config mainly used in DAO.
    public fun new_consensus_config(
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
    ): ConsensusConfig {
        assert!(uncle_rate_target > 0, error::invalid_argument(EINVALID_ARGUMENT));
        assert!(base_block_time_target > 0, error::invalid_argument(EINVALID_ARGUMENT));
        assert!(base_reward_per_block > 0, error::invalid_argument(EINVALID_ARGUMENT));
        assert!(epoch_block_count > 0, error::invalid_argument(EINVALID_ARGUMENT));
        assert!(base_block_difficulty_window > 0, error::invalid_argument(EINVALID_ARGUMENT));
        // base_reward_per_uncle_percent can been zero.
        assert!(min_block_time_target > 0, error::invalid_argument(EINVALID_ARGUMENT));
        assert!(max_block_time_target >= min_block_time_target, error::invalid_argument(EINVALID_ARGUMENT));

        ConsensusConfig {
            uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            epoch_block_count,
            base_block_difficulty_window,
            base_reward_per_uncle_percent,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy,
        }
    }

    /// Get config data.
    public fun get_config(): ConsensusConfig {
        on_chain_config::get_by_address<ConsensusConfig>(system_addresses::get_starcoin_framework())
    }

    /// Get uncle_rate_target
    public fun uncle_rate_target(config: &ConsensusConfig): u64 {
        config.uncle_rate_target
    }

    /// Get base_block_time_target
    public fun base_block_time_target(config: &ConsensusConfig): u64 {
        config.base_block_time_target
    }

    /// Get base_reward_per_block
    public fun base_reward_per_block(config: &ConsensusConfig): u128 {
        config.base_reward_per_block
    }

    /// Get epoch_block_count
    public fun epoch_block_count(config: &ConsensusConfig): u64 {
        config.epoch_block_count
    }

    /// Get base_block_difficulty_window
    public fun base_block_difficulty_window(config: &ConsensusConfig): u64 {
        config.base_block_difficulty_window
    }

    /// Get base_reward_per_uncle_percent
    public fun base_reward_per_uncle_percent(config: &ConsensusConfig): u64 {
        config.base_reward_per_uncle_percent
    }

    /// Get min_block_time_target
    public fun min_block_time_target(config: &ConsensusConfig): u64 {
        config.min_block_time_target
    }

    /// Get max_block_time_target
    public fun max_block_time_target(config: &ConsensusConfig): u64 {
        config.max_block_time_target
    }

    /// Get base_max_uncles_per_block
    public fun base_max_uncles_per_block(config: &ConsensusConfig): u64 {
        config.base_max_uncles_per_block
    }

    /// Get base_block_gas_limit
    public fun base_block_gas_limit(config: &ConsensusConfig): u64 {
        config.base_block_gas_limit
    }

    /// Get strategy
    public fun strategy(config: &ConsensusConfig): u8 {
        config.strategy
    }

    /// Compute block reward given the `new_epoch_block_time_target`.
    public fun compute_reward_per_block(new_epoch_block_time_target: u64): u128 {
        let config = get_config();
        do_compute_reward_per_block(&config, new_epoch_block_time_target)
    }

    /// Compute block reward given the `new_epoch_block_time_target`, and the consensus config.
    public fun do_compute_reward_per_block(config: &ConsensusConfig, new_epoch_block_time_target: u64): u128 {
        math128::mul_div(
            config.base_reward_per_block,
            (new_epoch_block_time_target as u128),
            (config.base_block_time_target as u128)
        )
    }

}