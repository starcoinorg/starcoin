address 0x1 {

module Consensus {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::FixedPoint32;

    struct Consensus {
        uncle_rate_target: FixedPoint32::FixedPoint32,
        epoch_time_target: u64,
        reward_half_time_target:u64,                
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == Config::default_config_address(), 1);

        Config::publish_new_config<Self::Consensus>(
            account,
            Consensus { 
                uncle_rate_target: FixedPoint32::create_from_rational(8,0) ,
                epoch_time_target :1209600, // two weeks in seconds
                reward_half_time_target: 126144000, // four years in seconds
            },
        );
    }

    public fun set_uncle_rate_target(account: &signer, numerator: u64, denominator: u64) {
        let old_config = Config::get<Self::Consensus>();

        old_config.uncle_rate_target = FixedPoint32::create_from_rational(numerator,denominator);
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

    public fun uncle_rate_target(): FixedPoint32::FixedPoint32  {
        let current_config = Config::get<Self::Consensus>();
        *&current_config.uncle_rate_target
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
