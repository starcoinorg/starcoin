/// The module provide configuration for block reward.
module starcoin_framework::block_reward_config {

    use starcoin_framework::system_addresses;
    use starcoin_framework::on_chain_config;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    /// Reward configuration
    struct RewardConfig has copy, drop, store {
        /// how many blocks delay reward distribution.
        reward_delay: u64,
    }

    const EINVALID_ARGUMENT: u64 = 18;

    /// Module initialization.
    public fun initialize(account: &signer, reward_delay: u64) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(account);

        on_chain_config::publish_new_config<Self::RewardConfig>(
            account,
            new_reward_config(reward_delay)
        );
    }

    spec initialize {
        use std::signer;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<on_chain_config::Config<RewardConfig>>(signer::address_of(account));
        include on_chain_config::PublishNewConfigAbortsIf<RewardConfig>;
        include on_chain_config::PublishNewConfigEnsures<RewardConfig>;
    }

    /// Create a new reward config mainly used in DAO.
    public fun new_reward_config(reward_delay: u64): RewardConfig {
        RewardConfig { reward_delay: reward_delay }
    }

    spec new_reward_config {}

    /// Get reward configuration.
    public fun get_reward_config(): RewardConfig {
        on_chain_config::get_by_address<RewardConfig>(system_addresses::get_starcoin_framework())
    }

    spec get_reward_config {
        include GetRewardConfigAbortsIf;
    }

    spec schema GetRewardConfigAbortsIf {
        aborts_if !exists<on_chain_config::Config<RewardConfig>>(system_addresses::get_starcoin_framework());
    }

    /// Get reward delay.
    public fun reward_delay(): u64 {
        let reward_config = get_reward_config();
        reward_config.reward_delay
    }

    spec reward_delay {
        aborts_if !exists<on_chain_config::Config<RewardConfig>>(system_addresses::get_starcoin_framework());
    }
}