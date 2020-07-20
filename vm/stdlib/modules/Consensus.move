address 0x1 {

module Consensus {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Event;

    struct Consensus {
        uncle_rate_target: u64,
        epoch_time_target: u64,
        reward_half_epoch:u64,
        block_difficulty_window: u64,
        reward_per_uncle_percent: u64,
        min_time_target: u64,
        max_uncles_per_block: u64,
    }

    resource struct Epoch {
        epoch_number: u64,
        epoch_start_time: u64,
        start_number: u64,
        end_number: u64,
        block_time_target: u64,
        reward_per_epoch: u64,
        reward_per_block: u64,
        new_epoch_events: Event::EventHandle<NewEpochEvent>,
    }

    struct NewEpochEvent {
        epoch_number: u64,
        epoch_start_time: u64,
        start_number: u64,
        end_number: u64,
        block_time_target: u64,
        reward_per_epoch: u64,
        reward_per_block: u64,
    }

    resource struct EpochData {
        uncles: u64,
        total_reward: u64,
    }

    public fun initialize(account: &signer,uncle_rate_target:u64,epoch_time_target: u64,
        reward_half_epoch: u64,init_block_time_target: u64, block_difficulty_window: u64,
        init_reward_per_epoch: u64, reward_per_uncle_percent: u64,
        min_time_target:u64, max_uncles_per_block:u64) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        assert(uncle_rate_target > 0, 2);
        assert(epoch_time_target > 0, 3);
        assert(reward_half_epoch > 0, 4);
        assert(init_block_time_target > 0, 5);
        assert(block_difficulty_window > 0, 6);
        assert(init_reward_per_epoch > 0, 7);
        assert(reward_per_uncle_percent > 0, 8);
        assert(min_time_target > 0, 9);
        assert(max_uncles_per_block >= 0, 10);

        move_to<Epoch>(account, Epoch {
            epoch_number:0,
            epoch_start_time: 0,
            start_number: 0,
            end_number: 0,
            block_time_target: init_block_time_target,
            reward_per_epoch: init_reward_per_epoch,
            reward_per_block: 0,
            new_epoch_events: Event::new_event_handle<NewEpochEvent>(account),
        });

        move_to<EpochData>(account, EpochData {
            uncles: 0,
            total_reward: 0,
        });

        Config::publish_new_config<Self::Consensus>(
            account,
            Consensus { 
                uncle_rate_target: uncle_rate_target,//80
                epoch_time_target : epoch_time_target, // two weeks in seconds 1209600
                reward_half_epoch: reward_half_epoch,
                block_difficulty_window: block_difficulty_window,
                reward_per_uncle_percent: reward_per_uncle_percent,
                min_time_target: min_time_target,
                max_uncles_per_block: max_uncles_per_block,
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

    public fun min_time_target(): u64  {
        let current_config = get_config();
        current_config.min_time_target
    }

    public fun reward_half_epoch(): u64  {
        let current_config = get_config();
        current_config.reward_half_epoch
    }

    public fun reward_per_uncle_percent(): u64 {
        let current_config = get_config();
        current_config.reward_per_uncle_percent
    }

    public fun max_uncles_per_block():u64 {
        let current_config = get_config();
        current_config.max_uncles_per_block
    }

    fun block_difficulty_window(): u64 {
        let current_config = get_config();
        current_config.block_difficulty_window
    }

    fun reward_per_block(blocks:u64, reward_per_epoch: u64): u64 {
        let max_uncles = (blocks * Self::uncle_rate_target() * Self::reward_per_uncle_percent()) / (1000 * 100);
        let reward = reward_per_epoch / (max_uncles + blocks);
        reward
    }

    fun first_epoch(block_height: u64, block_time: u64) acquires Epoch {
        assert(block_height == 1, 333);
        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        let count = Self::epoch_time_target() / epoch_ref.block_time_target;
        assert(count > 1, 336);
        epoch_ref.epoch_start_time = block_time;
        epoch_ref.start_number = 1;
        epoch_ref.end_number = epoch_ref.start_number + count;
        epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
        epoch_ref.reward_per_block = Self::reward_per_block(count, epoch_ref.reward_per_epoch);
    }

    public fun adjust_epoch(account: &signer, block_height: u64, block_time: u64, uncles: u64): u64 acquires Epoch, EpochData {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 33);
        assert(Self::max_uncles_per_block() >= uncles, 339);
        if (block_height == 1) {
            assert(uncles == 0, 334);
            Self::first_epoch(block_height, block_time);
        } else {
            let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
            let epoch_data = borrow_global_mut<EpochData>(CoreAddresses::GENESIS_ACCOUNT());
            if (block_height < epoch_ref.end_number) {
                epoch_data.uncles = epoch_data.uncles + uncles;
            } else {
                assert(uncles == 0, 334);
                assert(block_time > epoch_ref.epoch_start_time, 335);
                let total_time = block_time - epoch_ref.epoch_start_time;
                let total_uncles = epoch_data.uncles;
                let blocks = epoch_ref.end_number - epoch_ref.start_number;
                let avg_block_time = total_time / blocks;
                let uncles_rate = total_uncles * 1000 / blocks;
                let new_epoch_block_time_target = (1000 + uncles_rate) * avg_block_time / (Self::uncle_rate_target() + 1000);
                let total_reward = epoch_data.total_reward;

                if (new_epoch_block_time_target < Self::min_time_target()) {
                    new_epoch_block_time_target = Self::min_time_target();
                };
                let new_epoch_blocks = if (total_time >= (Self::epoch_time_target() * 2)) {
                    1
                } else {
                    let new_epoch_time_target = Self::epoch_time_target() * 2 - total_time;
                    new_epoch_time_target / new_epoch_block_time_target
                };
                assert(new_epoch_blocks >= 1, 337);

                epoch_ref.epoch_start_time = block_time;
                epoch_data.uncles = uncles;
                epoch_ref.start_number = block_height;
                epoch_ref.end_number = block_height + new_epoch_blocks;
                epoch_ref.block_time_target = new_epoch_block_time_target;
                epoch_ref.epoch_number = epoch_ref.epoch_number + 1;

                let old_reward_per_epoch = epoch_ref.reward_per_epoch;
                let current_reward_per_epoch = if (epoch_ref.epoch_number % Self::reward_half_epoch() == 1) {
                    (epoch_ref.reward_per_epoch / 2)
                } else {
                    epoch_ref.reward_per_epoch
                };

                if ((old_reward_per_epoch + current_reward_per_epoch) > total_reward) {
                    epoch_ref.reward_per_epoch = (old_reward_per_epoch + current_reward_per_epoch) - total_reward;
                } else {
                    epoch_ref.reward_per_epoch = 0;
                };

                epoch_ref.reward_per_block = Self::reward_per_block(new_epoch_blocks, epoch_ref.reward_per_epoch);
            }
        };

        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        let reward = epoch_ref.reward_per_block + (epoch_ref.reward_per_block * Self::reward_per_uncle_percent() * uncles / 100);

        let epoch_data = borrow_global_mut<EpochData>(CoreAddresses::GENESIS_ACCOUNT());
        if (block_height == epoch_ref.start_number) {
            epoch_data.total_reward = reward;
        } else {
            epoch_data.total_reward = epoch_data.total_reward + reward;
        };

        Event::emit_event(
            &mut epoch_ref.new_epoch_events,
            NewEpochEvent {
                epoch_number: epoch_ref.epoch_number,
                epoch_start_time: epoch_ref.epoch_start_time,
                start_number: epoch_ref.start_number,
                end_number: epoch_ref.end_number,
                block_time_target: epoch_ref.block_time_target,
                reward_per_epoch: epoch_ref.reward_per_epoch,
                reward_per_block: epoch_ref.reward_per_block,
            }
        );

        reward
    }

    public fun epoch_start_time(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.epoch_start_time
    }

    public fun uncles(): u64 acquires EpochData {
        let epoch_data = borrow_global<EpochData>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_data.uncles
    }

    public fun start_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.start_number
    }

    public fun end_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.end_number
    }

    public fun epoch_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.epoch_number
    }

    public fun block_time_target(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.block_time_target
    }

    public fun reward_per_epoch(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ACCOUNT());
        epoch_ref.reward_per_epoch
    }
}

}
