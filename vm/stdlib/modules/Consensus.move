address 0x1 {

module Consensus {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct Consensus {
        uncle_rate_target: u64,
        epoch_time_target: u64,
        reward_half_time_target:u64,                
    }

    resource struct Epoch {
        epoch_start_time: u64,
        uncles: u64,
        start_number: u64,
        end_number: u64,
        time_target: u64,
        reward: u64,
    }

    public fun initialize(account: &signer,uncle_rate_target:u64,epoch_time_target: u64,reward_half_time_target: u64,init_block_time_target: u64) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

        move_to<Epoch>(account, Epoch {
             epoch_start_time: 0,
             uncles: 0,
             start_number: 0,
             end_number: 10,
             time_target: init_block_time_target,
             reward: 0,
         });

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
        let old_config = Config::get<Self::Consensus>(account);

        old_config.uncle_rate_target = uncle_rate_target;
        Config::set<Self::Consensus>(
            account,
            old_config,    
        );
    }

    public fun set_epoch_time_target(account: &signer, epoch_time_target: u64) {
        let old_config = Config::get<Self::Consensus>(account);

        old_config.epoch_time_target = epoch_time_target;
        Config::set<Self::Consensus>(
            account,
            old_config,    
        );
    }

    public fun set_reward_half_time_target(account: &signer, reward_half_time_target: u64) {
        let old_config = Config::get<Self::Consensus>(account);

        old_config.reward_half_time_target = reward_half_time_target;
        Config::set<Self::Consensus>(
            account,
            old_config,    
        );
    }
    
    public fun get_config(): Consensus{
        Config::get_by_address<Consensus>(CoreAddresses::GENESIS_ACCOUNT())
    }

    public fun uncle_rate_target(): u64  {
        let current_config = get_config();
        current_config.uncle_rate_target
    }

    public fun epoch_time_target(): u64  {
        let current_config = get_config();
        current_config.epoch_time_target
    }

    public fun reward_half_time_target(): u64  {
        let current_config = get_config();
        current_config.reward_half_time_target
    }

    fun first_epoch(block_height: u64, block_time: u64) acquires Epoch {
        assert(block_height == 1, 333);
        let count = Self::epoch_time_target() / block_time;
        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.epoch_start_time = block_time;
        epoch_ref.start_number = 1;
        epoch_ref.end_number = count;
    }

    public fun adjust_epoch(account: &signer, block_height: u64, block_time: u64, uncles: u64) acquires Epoch {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 33);
        if (block_height == 1) {
            assert(uncles == 0, 334);
            Self::first_epoch(block_height, block_time);
        } else {
            let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
            if (block_height < epoch_ref.end_number) {
                epoch_ref.uncles = epoch_ref.uncles + uncles;
            } else {
                //TODO:
                epoch_ref.uncles = 0;
                epoch_ref.epoch_start_time = block_time;
                epoch_ref.start_number = block_height + 1;
                epoch_ref.end_number = block_height + 10;
            }
        }
    }
}

}
