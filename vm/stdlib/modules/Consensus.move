address 0x1 {

module Consensus {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct Consensus {
        uncle_rate_target: u64,
        epoch_time_target: u64,
        reward_half_epoch:u64,
        block_window: u64,
        only_current_epoch: bool,
        reward_per_uncle_percent: u64,
    }

    resource struct Epoch {
        epoch_number: u64,
        epoch_start_time: u64,
        uncles: u64,
        start_number: u64,
        end_number: u64,
        time_target: u64,
        window: u64,
        reward_per_epoch: u64,
    }

    public fun initialize(account: &signer,uncle_rate_target:u64,epoch_time_target: u64,
        reward_half_epoch: u64,init_block_time_target: u64, block_window: u64,
        only_current_epoch: bool, init_reward_per_epoch: u64, reward_per_uncle_percent: u64) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        assert(init_block_time_target > 0, 2);
        assert(init_reward_per_epoch > 0, 3);

        move_to<Epoch>(account, Epoch {
            epoch_number:0,
            epoch_start_time: 0,
            uncles: 0,
            start_number: 0,
            end_number: 0,
            time_target: init_block_time_target,
            window: 0,
            reward_per_epoch: init_reward_per_epoch,
        });

        Config::publish_new_config<Self::Consensus>(
            account,
            Consensus { 
                uncle_rate_target: uncle_rate_target,//80
                epoch_time_target : epoch_time_target, // two weeks in seconds 1209600
                reward_half_epoch: reward_half_epoch,
                block_window: block_window,
                only_current_epoch: only_current_epoch,
                reward_per_uncle_percent: reward_per_uncle_percent,
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

    public fun set_reward_half_epoch(account: &signer, reward_half_epoch: u64) {
        let old_config = Config::get<Self::Consensus>(account);

        old_config.reward_half_epoch = reward_half_epoch;
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

    public fun reward_half_epoch(): u64  {
        let current_config = get_config();
        current_config.reward_half_epoch
    }

    public fun reward_per_uncle_percent(): u64 {
        let current_config = get_config();
        current_config.reward_per_uncle_percent
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

    fun reward_per_block(): u64 acquires Epoch {
        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        let blocks = epoch_ref.end_number - epoch_ref.start_number + 1;
        let max_uncles = (blocks * Self::uncle_rate_target() * Self::reward_per_uncle_percent()) / (1000 * 100);
        let reward = epoch_ref.reward_per_epoch / (max_uncles + blocks);
        reward
    }

    fun reward_per_uncle(): u64 acquires Epoch {
        let reward = Self::reward_per_block() * Self::reward_per_uncle_percent();
        reward
    }

    fun first_epoch(account: &signer, block_height: u64, block_time: u64) acquires Epoch {
        assert(block_height == 1, 333);
        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        let count = Self::epoch_time_target() / epoch_ref.time_target;
        assert(count > 1, 336);
        epoch_ref.epoch_start_time = block_time;
        epoch_ref.start_number = 1;
        epoch_ref.end_number = count;
        epoch_ref.window = Self::block_window(account, 1, block_height);
        epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
    }

    public fun adjust_epoch(account: &signer, block_height: u64, block_time: u64, uncles: u64): u64 acquires Epoch {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 33);
        if (block_height == 1) {
            assert(uncles == 0, 334);
            Self::first_epoch(account, block_height, block_time);
        } else {
            let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
            if (block_height <= epoch_ref.end_number) {
                epoch_ref.uncles = epoch_ref.uncles + uncles;
                if (block_height == epoch_ref.end_number) {
                    epoch_ref.window = Self::block_window(account, 0, block_height);
                } else {
                    epoch_ref.window = Self::block_window(account, block_height - epoch_ref.start_number + 1, block_height);
                }
            } else {
                assert(block_time > epoch_ref.epoch_start_time, 335);
                let total_time = block_time - epoch_ref.epoch_start_time;
                let total_uncles = epoch_ref.uncles;
                let blocks = epoch_ref.end_number - epoch_ref.start_number + 1;
                let avg_block_time = total_time / blocks;
                let uncles_rate = total_uncles * 1000 / blocks;
                let new_epoch_block_time_target = (1000 + uncles_rate) * avg_block_time / (Self::uncle_rate_target() + 1000);
                if (new_epoch_block_time_target < 10) {
                    new_epoch_block_time_target = 10;
                };
                let new_epoch_time_target = Self::epoch_time_target() * 2 - total_time;
                let new_epoch_blocks = new_epoch_time_target / new_epoch_block_time_target;
                assert(new_epoch_blocks > 1, 337);

                epoch_ref.epoch_start_time = block_time;
                epoch_ref.uncles = uncles;
                epoch_ref.start_number = block_height;
                epoch_ref.end_number = block_height + new_epoch_blocks;
                epoch_ref.time_target = new_epoch_block_time_target;
                epoch_ref.window = Self::block_window(account, 1, block_height);
                epoch_ref.epoch_number = epoch_ref.epoch_number + 1;

                if (epoch_ref.epoch_number % Self::reward_half_epoch() == 0) {
                    epoch_ref.reward_per_epoch = (epoch_ref.reward_per_epoch / 2);
                }
            }
        };

        let reward = Self::reward_per_block() + (Self::reward_per_uncle() * uncles);
        reward
    }

    public fun epoch_start_time(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.epoch_start_time
    }

    public fun uncles(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.uncles
    }

    public fun start_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.start_number
    }

    public fun end_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.end_number
    }

    public fun time_target(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.time_target
    }

    public fun window(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.window
    }

    public fun reward_per_epoch(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.reward_per_epoch
    }
}

}
