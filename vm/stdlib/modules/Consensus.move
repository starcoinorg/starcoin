address 0x1 {

module Consensus {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct Consensus {
        uncle_rate_target: u64,
        epoch_time_target: u64,
        reward_half_time_target:u64,
        block_window: u64,
        only_current_epoch: bool,
    }

    resource struct Epoch {
        epoch_start_time: u64,
        uncles: u64,
        start_number: u64,
        end_number: u64,
        time_target: u64,
        window: u64,
        reward: u64,
    }

    public fun initialize(account: &signer,uncle_rate_target:u64,epoch_time_target: u64,reward_half_time_target: u64,init_block_time_target: u64, block_window: u64, only_current_epoch: bool) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

        move_to<Epoch>(account, Epoch {
             epoch_start_time: 0,
             uncles: 0,
             start_number: 0,
             end_number: 0,
             time_target: init_block_time_target,
             window: 0,
             reward: 0,
         });

        Config::publish_new_config<Self::Consensus>(
            account,
            Consensus { 
                uncle_rate_target: uncle_rate_target,//80
                epoch_time_target : epoch_time_target, // two weeks in seconds 1209600
                reward_half_time_target: reward_half_time_target, // four years in seconds 126144000
                block_window: block_window,
                only_current_epoch: only_current_epoch,
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

    fun block_window(account: &signer, gap: u64, height: u64): u64 {
        let current_config = Config::get<Self::Consensus>(account);
        if (current_config.only_current_epoch) {
            if (gap <= current_config.block_window) {
                gap
            } else {
                current_config.block_window
            }
        } else {
            if (height < current_config.block_window) {
                height
            } else {
                current_config.block_window
            }
        }
    }

    fun first_epoch(account: &signer, block_height: u64, block_time: u64) acquires Epoch {
        assert(block_height == 1, 333);
        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        let count = Self::epoch_time_target() / epoch_ref.time_target;
        epoch_ref.epoch_start_time = block_time;
        epoch_ref.start_number = 1;
        epoch_ref.end_number = count;
        epoch_ref.window = Self::block_window(account, 1, block_height);
    }

    //TODO: has bug
    public fun adjust_epoch(account: &signer, block_height: u64, block_time: u64, uncles: u64) acquires Epoch {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 33);
        if (block_height == 1) {
            assert(uncles == 0, 334);
            Self::first_epoch(account, block_height, block_time);
        } else {
            let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
            if (block_height < epoch_ref.end_number) {
                epoch_ref.uncles = epoch_ref.uncles + uncles;
                epoch_ref.window = Self::block_window(account, block_height - epoch_ref.start_number + 1, block_height);
            } else {
                //TODO:
                assert(block_time > epoch_ref.epoch_start_time, 335);
                let total_time = block_time - epoch_ref.epoch_start_time;
                let total_uncles = epoch_ref.uncles + uncles;
                let avg_block_time = total_time / (epoch_ref.end_number - epoch_ref.start_number);
                let uncles_rate = total_uncles * 1000 / (epoch_ref.end_number - epoch_ref.start_number);
                let new_epoch_block_time_target = (1000 + Self::uncle_rate_target()) * avg_block_time / (uncles_rate + 1000);
                let new_epoch_time_target = Self::epoch_time_target() * 2 - total_time;
                let new_epoch_blocks = new_epoch_time_target / new_epoch_block_time_target;
                assert(new_epoch_blocks > 1, 336);

                epoch_ref.epoch_start_time = block_time;
                epoch_ref.uncles = 0;
                epoch_ref.start_number = block_height + 1;
                epoch_ref.end_number = block_height + new_epoch_blocks;
                epoch_ref.time_target = new_epoch_block_time_target;
                epoch_ref.window = Self::block_window(account, 0, block_height);
            }
        }
    }

    public fun current_epoch_info(): (u64, u64, u64) acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        let start_number = epoch_ref.start_number;
        let end_number = epoch_ref.end_number;
        let time_target = epoch_ref.time_target;
        (start_number, end_number, time_target)
    }
}

}
