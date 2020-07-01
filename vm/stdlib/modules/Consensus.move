address 0x1 {

module Consensus {
    use 0x1::Config;
    use 0x1::Signer;

    struct Consensus {
        uncle_rate_target: u64,
        epoch_time_target: u64,
        reward_half_time_target:u64,                
    }

    public fun initialize(account: &signer,uncle_rate_target:u64,epoch_time_target: u64,reward_half_time_target: u64) {
        assert(Signer::address_of(account) == Config::default_config_address(), 1);

        Config::publish_new_config<Self::Consensus>(
            account,
            Consensus { 
                uncle_rate_target: uncle_rate_target,//80
                epoch_time_target : epoch_time_target, // two weeks in seconds 1209600
                reward_half_time_target: reward_half_time_target, // four years in seconds 126144000
            },
        );
    }

    public fun set_uncle_rate_target(account: &signer, uncle_rate_target:u64) {
        let old_config = Config::get<Self::Consensus>();

        old_config.uncle_rate_target = uncle_rate_target;
        Config::set<Self::Consensus>(
            account,
            old_config,    
        );
    }

    public fun set_epoch_time_target(account: &signer, epoch_time_target: u64) {
        let old_config = Config::get<Self::Consensus>();

        old_config.epoch_time_target = epoch_time_target;
        Config::set<Self::Consensus>(
            account,
            old_config,    
        );
    }

    public fun set_reward_half_time_target(account: &signer, reward_half_time_target: u64) {
        let old_config = Config::get<Self::Consensus>();

        old_config.reward_half_time_target = reward_half_time_target;
        Config::set<Self::Consensus>(
            account,
            old_config,    
        );
    }

    public fun uncle_rate_target(): u64  {
        let current_config = Config::get<Self::Consensus>();
        current_config.uncle_rate_target
    }

    public fun epoch_time_target(): u64  {
        let current_config = Config::get<Self::Consensus>();
        current_config.epoch_time_target
    }

    public fun reward_half_time_target(): u64  {
        let current_config = Config::get<Self::Consensus>();
        current_config.reward_half_time_target
    }

}

}
