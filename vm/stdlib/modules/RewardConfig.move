address 0x1{
// Block reward config
// TODO this may be as a coin mint strategy, support any type coin.
module RewardConfig {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct RewardConfig {
        reward_halving_interval: u64,
        reward_base: u64,
        reward_delay: u64,
    }

    public fun initialize(config_account: &signer, reward_halving_interval: u64, reward_base: u64, reward_delay: u64) {
        assert(Signer::address_of(config_account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        assert(reward_halving_interval > 0, 6106);
        assert(reward_base > 0, 6107);
        assert(reward_delay > 0, 6108);

        Config::publish_new_config<RewardConfig>(
            config_account,
            RewardConfig {
                reward_halving_interval,
                reward_base,
                reward_delay,
            },
        );
    }

    public fun get_config(): RewardConfig{
        Config::get_by_address<RewardConfig>(CoreAddresses::GENESIS_ACCOUNT())
    }

    //Calculate reward by block height.
    public fun reward_coin(block_height: u64): u64  {
        let current_config = get_config();
        let halving = current_config.reward_halving_interval;
        let reward = current_config.reward_base;
        let times = block_height / halving;
        let index = 0;

        while (index < times) {
            if (reward == 0) {
                break
            };
            reward = reward / 2;
            index = index + 1;
        };

        reward
    }

    public fun reward_halving_interval(): u64  {
        let current_config = get_config();
        current_config.reward_halving_interval
    }

    public fun reward_base(): u64  {
        let current_config = get_config();
        current_config.reward_base
    }

    public fun reward_delay(): u64  {
        let current_config = get_config();
        current_config.reward_delay
    }

}
}