address 0x1 {
module Consensus {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::ErrorCode;
    use 0x1::Timestamp;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict;
    }

    const THOUSAND: u64 = 1000;
    const THOUSAND_U128: u128 = 1000;
    const HUNDRED: u64 = 100;

    struct Consensus {
        uncle_rate_target: u64,
        init_block_time_target: u64,
        init_reward_per_block: u128,
        reward_per_uncle_percent: u64,
        epoch_block_count: u64,
        block_difficulty_window: u64,
        min_block_time_target: u64,
        max_block_time_target: u64,
        max_uncles_per_block: u64,
    }

    resource struct Epoch {
        epoch_number: u64,
        epoch_start_time: u64,
        start_number: u64,
        end_number: u64,
        block_time_target: u64,
        reward_per_block: u128,
        new_epoch_events: Event::EventHandle<NewEpochEvent>,
    }

    struct NewEpochEvent {
        epoch_number: u64,
        epoch_start_time: u64,
        start_number: u64,
        end_number: u64,
        block_time_target: u64,
        reward_per_block: u128,
        previous_epoch_total_reward: u128,
    }

    resource struct EpochData {
        uncles: u64,
        total_reward: u128,
    }

    fun MAX_UNCLES_PER_BLOCK_IS_WRONG(): u64 {
        ErrorCode::ECODE_BASE() + 1
    }

    fun UNCLES_IS_NOT_ZERO(): u64 {
        ErrorCode::ECODE_BASE() + 2
    }

    public fun initialize(
        account: &signer,
        uncle_rate_target: u64,
        epoch_block_count: u64,
        init_block_time_target: u64,
        block_difficulty_window: u64,
        init_reward_per_block: u128,
        reward_per_uncle_percent: u64,
        min_block_time_target: u64,
        max_block_time_target: u64,
        max_uncles_per_block: u64,
    ) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            ErrorCode::ENOT_GENESIS_ACCOUNT(),
        );
        assert(uncle_rate_target > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(epoch_block_count > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(init_reward_per_block > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(init_block_time_target > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(block_difficulty_window > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(reward_per_uncle_percent > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(min_block_time_target > 0, ErrorCode::EINVALID_ARGUMENT());
        assert(max_uncles_per_block >= 0, ErrorCode::EINVALID_ARGUMENT());
        move_to<Epoch>(
            account,
            Epoch {
                epoch_number: 0,
                epoch_start_time: Timestamp::now_seconds(),
                start_number: 0,
                end_number: epoch_block_count,
                block_time_target: init_block_time_target,
                reward_per_block: init_reward_per_block,
                new_epoch_events: Event::new_event_handle<NewEpochEvent>(account),
            },
        );
        move_to<EpochData>(account, EpochData { uncles: 0, total_reward: 0 });
        Config::publish_new_config<Self::Consensus>(
            account,
            Consensus {
                uncle_rate_target,
                init_block_time_target,
                init_reward_per_block,
                epoch_block_count,
                block_difficulty_window,
                reward_per_uncle_percent,
                min_block_time_target,
                max_block_time_target,
                max_uncles_per_block,
            },
        );
    }

    public fun set_uncle_rate_target(account: &signer, uncle_rate_target: u64) {
        let old_config = Config::get_by_address<Self::Consensus>(Signer::address_of(account));
        old_config.uncle_rate_target = uncle_rate_target;
        Config::set<Self::Consensus>(account, old_config);
    }

    public fun set_epoch_block_count(account: &signer, epoch_block_count: u64) {
        let old_config = Config::get_by_address<Self::Consensus>(Signer::address_of(account));
        old_config.epoch_block_count = epoch_block_count;
        Config::set<Self::Consensus>(account, old_config);
    }

    public fun set_min_block_time_target(account: &signer, min_block_time_target: u64) {
        let old_config = Config::get_by_address<Self::Consensus>(Signer::address_of(account));
        old_config.min_block_time_target = min_block_time_target;
        Config::set<Self::Consensus>(account, old_config);
    }

    fun get_config(): Consensus {
        Config::get_by_address<Consensus>(CoreAddresses::GENESIS_ADDRESS())
    }

    public fun uncle_rate_target(): u64 {
        let current_config = get_config();
        current_config.uncle_rate_target
    }

    public fun epoch_block_count(): u64 {
        let current_config = get_config();
        current_config.epoch_block_count
    }

    public fun init_block_time_target(): u64 {
        let current_config = get_config();
        current_config.init_block_time_target
    }

    public fun min_block_time_target(): u64 {
        let current_config = get_config();
        current_config.min_block_time_target
    }

    public fun max_block_time_target(): u64 {
        let current_config = get_config();
        current_config.max_block_time_target
    }

    public fun reward_per_uncle_percent(): u64 {
        let current_config = get_config();
        current_config.reward_per_uncle_percent
    }

    public fun max_uncles_per_block(): u64 {
        let current_config = get_config();
        current_config.max_uncles_per_block
    }

    public fun block_difficulty_window(): u64 {
        let current_config = get_config();
        current_config.block_difficulty_window
    }

    public fun compute_reward_per_block(new_epoch_block_time_target: u64): u128 {
        let current_config = get_config();
        current_config.init_reward_per_block *
            (new_epoch_block_time_target as u128) * THOUSAND_U128 /
                (current_config.init_block_time_target as u128) / THOUSAND_U128
    }

    public fun adjust_epoch(account: &signer, block_number: u64, now: u64, uncles: u64): u128
    acquires Epoch, EpochData {
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            ErrorCode::ENOT_GENESIS_ACCOUNT(),
        );
        assert(Self::max_uncles_per_block() >= uncles, MAX_UNCLES_PER_BLOCK_IS_WRONG());
        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        let epoch_data = borrow_global_mut<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        let (new_epoch, reward_per_block) = if (block_number < epoch_ref.end_number) {
            (false, epoch_ref.reward_per_block)
        } else if (block_number == epoch_ref.end_number) {
            //start a new epoch
            assert(uncles == 0, UNCLES_IS_NOT_ZERO());
            let config = get_config();
            let total_time = now - epoch_ref.epoch_start_time;
            let total_uncles = epoch_data.uncles;
            let blocks = epoch_ref.end_number - epoch_ref.start_number;
            let avg_block_time = total_time / blocks;
            let uncles_rate = total_uncles * THOUSAND / blocks;
            let new_epoch_block_time_target = (THOUSAND + uncles_rate) * avg_block_time /
                (config.uncle_rate_target + THOUSAND);
            if (new_epoch_block_time_target < config.min_block_time_target) {
                new_epoch_block_time_target = config.min_block_time_target;
            };
            if (new_epoch_block_time_target > config.max_block_time_target) {
                new_epoch_block_time_target = config.max_block_time_target;
            };
            let new_reward_per_block = Self::compute_reward_per_block(new_epoch_block_time_target);
            epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
            epoch_ref.epoch_start_time = now;
            epoch_data.uncles = uncles;
            epoch_ref.start_number = block_number;
            epoch_ref.end_number = block_number + config.epoch_block_count;
            epoch_ref.block_time_target = new_epoch_block_time_target;
            epoch_ref.reward_per_block = new_reward_per_block;
            emit_epoch_event(epoch_ref, epoch_data.total_reward);
            (true, new_reward_per_block)
        } else {
            //This should never happend.
            abort ErrorCode::EUNREACHABLE()
        };
        let reward = reward_per_block +
            reward_per_block * (Self::reward_per_uncle_percent() as u128) * (uncles as u128) / 100;
        update_epoch_data(epoch_data, new_epoch, reward, uncles);
        reward
    }

    fun update_epoch_data(epoch_data: &mut EpochData, new_epoch: bool, reward: u128, uncles: u64) {
        if (new_epoch) {
            epoch_data.total_reward = reward;
            epoch_data.uncles = uncles;
        } else {
            epoch_data.total_reward = epoch_data.total_reward + reward;
            epoch_data.uncles = epoch_data.uncles + uncles;
        }
    }

    fun emit_epoch_event(epoch_ref: &mut Epoch, previous_epoch_total_reward: u128) {
        Event::emit_event(
            &mut epoch_ref.new_epoch_events,
            NewEpochEvent {
                epoch_number: epoch_ref.epoch_number,
                epoch_start_time: epoch_ref.epoch_start_time,
                start_number: epoch_ref.start_number,
                end_number: epoch_ref.end_number,
                block_time_target: epoch_ref.block_time_target,
                reward_per_block: epoch_ref.reward_per_block,
                previous_epoch_total_reward,
            },
        );
    }

    public fun epoch_start_time(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.epoch_start_time
    }

    public fun uncles(): u64 acquires EpochData {
        let epoch_data = borrow_global<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        epoch_data.uncles
    }

    public fun epoch_start_block_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.start_number
    }

    public fun epoch_end_block_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.end_number
    }

    public fun epoch_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.epoch_number
    }

    public fun block_time_target(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.block_time_target
    }
}
}