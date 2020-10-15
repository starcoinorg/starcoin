address 0x1 {

module RewardConfig {
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::ErrorCode;
    use 0x1::Config;

    struct RewardConfig {
        reward_delay: u64,
    }

    public fun initialize(account: &signer, reward_delay: u64) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        Config::publish_new_config<Self::RewardConfig>(
            account,
            new_reward_config(reward_delay)
        );
    }

    public fun new_reward_config(reward_delay: u64) : RewardConfig {
        RewardConfig {reward_delay: reward_delay}
    }

    public fun get_reward_config(): RewardConfig {
        Config::get_by_address<RewardConfig>(CoreAddresses::GENESIS_ADDRESS())
    }

    public fun reward_delay() :u64 {
        let reward_config = get_reward_config();
        reward_config.reward_delay
    }
}
}