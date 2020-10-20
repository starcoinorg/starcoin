address 0x1 {

module RewardConfig {
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Config;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    struct RewardConfig {
        reward_delay: u64,
    }

    const EINVALID_ARGUMENT: u64 = 18;

    public fun initialize(account: &signer, reward_delay: u64) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);

        Config::publish_new_config<Self::RewardConfig>(
            account,
            new_reward_config(reward_delay)
        );
    }

    spec fun initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        aborts_if exists<Config::Config<RewardConfig>>(Signer::spec_address_of(account));
        include Config::PublishNewConfigAbortsIf<RewardConfig>;
        include Config::PublishNewConfigEnsures<RewardConfig>;
    }

    public fun new_reward_config(reward_delay: u64) : RewardConfig {
        RewardConfig {reward_delay: reward_delay}
    }

    spec fun new_reward_config {}

    public fun get_reward_config(): RewardConfig {
        Config::get_by_address<RewardConfig>(CoreAddresses::GENESIS_ADDRESS())
    }

    spec fun get_reward_config {
        include GetRewardConfigAbortsIf;
    }

    spec schema GetRewardConfigAbortsIf {
        aborts_if !exists<Config::Config<RewardConfig>>(CoreAddresses::GENESIS_ADDRESS());
    }

    public fun reward_delay() :u64 {
        let reward_config = get_reward_config();
        reward_config.reward_delay
    }

    spec fun reward_delay {
        aborts_if !exists<Config::Config<RewardConfig>>(CoreAddresses::GENESIS_ADDRESS());
    }
}
}