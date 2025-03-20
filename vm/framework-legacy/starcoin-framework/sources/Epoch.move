address StarcoinFramework {
/// The module provide epoch functionality for starcoin.
module Epoch {
    use StarcoinFramework::Config;
    use StarcoinFramework::Signer;
    use StarcoinFramework::CoreAddresses;

    use StarcoinFramework::Event;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Math;
    use StarcoinFramework::Option;
    use StarcoinFramework::ConsensusConfig::{Self, ConsensusConfig};

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Current epoch info.
    struct Epoch has key {
        /// Number of current epoch
        number: u64,
        /// Start time of current epoch
        start_time: u64,
        /// Start block's number of current epoch
        start_block_number: u64,
        /// End block's number of current epoch
        end_block_number: u64,
        /// Average target time to calculate a block's difficulty in current epoch
        block_time_target: u64,
        /// Rewards per block in current epoch
        reward_per_block: u128,
        /// Percentage of `reward_per_block` to reward a uncle block in current epoch
        reward_per_uncle_percent: u64,
        /// How many ancestor blocks which use to calculate next block's difficulty in current epoch
        block_difficulty_window: u64,
        /// Maximum number of uncle block per block in current epoch
        max_uncles_per_block: u64,
        /// Maximum gases per block in current epoch
        block_gas_limit: u64,
        /// Strategy to calculate difficulty in current epoch
        strategy: u8,
        /// Switch Epoch Event
        new_epoch_events: Event::EventHandle<NewEpochEvent>,
    }

    /// New epoch event.
    struct NewEpochEvent has drop, store {
        /// Epoch::number
        number: u64,
        /// Epoch::start_time
        start_time: u64,
        /// Epoch::start_block_number
        start_block_number: u64,
        /// Epoch::end_block_number
        end_block_number: u64,
        /// Epoch::block_time_target
        block_time_target: u64,
        /// Epoch::reward_per_block
        reward_per_block: u128,
        /// Total rewards during previous epoch
        previous_epoch_total_reward: u128,
    }

    /// Epoch data.
    struct EpochData has key {
        /// Up to now, Number of uncle block during current epoch
        uncles: u64,
        /// Up to now, Total rewards during current epoch
        total_reward: u128,
        /// Up to now, Total gases during current epoch
        total_gas: u128,
    }

    const THOUSAND: u64 = 1000;
    const THOUSAND_U128: u128 = 1000;
    const HUNDRED: u64 = 100;

    const EUNREACHABLE: u64 = 19;
    const EINVALID_UNCLES_COUNT: u64 = 101;

    /// Initialization of the module.
    public fun initialize(
        account: &signer,
    ) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);

        let config = ConsensusConfig::get_config();
        move_to<Epoch>(
            account,
            Epoch {
                number: 0,
                start_time: Timestamp::now_milliseconds(),
                start_block_number: 0,
                end_block_number: ConsensusConfig::epoch_block_count(&config),
                block_time_target: ConsensusConfig::base_block_time_target(&config),
                reward_per_block: ConsensusConfig::base_reward_per_block(&config),
                reward_per_uncle_percent: ConsensusConfig::base_reward_per_uncle_percent(&config),
                block_difficulty_window: ConsensusConfig::base_block_difficulty_window(&config),
                max_uncles_per_block: ConsensusConfig::base_max_uncles_per_block(&config),
                block_gas_limit: ConsensusConfig::base_block_gas_limit(&config),
                strategy: ConsensusConfig::strategy(&config),
                new_epoch_events: Event::new_event_handle<NewEpochEvent>(account),
            },
        );
        move_to<EpochData>(account, EpochData { uncles: 0, total_reward: 0, total_gas: 0 });
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if !exists<Config::Config<ConsensusConfig>>(CoreAddresses::SPEC_GENESIS_ADDRESS());

        aborts_if exists<Epoch>(Signer::address_of(account));
        aborts_if exists<EpochData>(Signer::address_of(account));
    }

    /// compute next block time_target.
    public fun compute_next_block_time_target(config: &ConsensusConfig, last_epoch_time_target: u64, epoch_start_time: u64, now_milli_second: u64, start_block_number: u64, end_block_number: u64, total_uncles: u64): u64 {
        let total_time = now_milli_second - epoch_start_time;
        let blocks = end_block_number - start_block_number;
        let avg_block_time = total_time / blocks;
        let uncles_rate = total_uncles * THOUSAND / blocks;
        let new_epoch_block_time_target = (THOUSAND + uncles_rate) * avg_block_time /
                (ConsensusConfig::uncle_rate_target(config) + THOUSAND);
        if (new_epoch_block_time_target > last_epoch_time_target * 2) {
            new_epoch_block_time_target = last_epoch_time_target * 2;
        };
        if (new_epoch_block_time_target < last_epoch_time_target / 2) {
            new_epoch_block_time_target = last_epoch_time_target / 2;
        };
        let min_block_time_target = ConsensusConfig::min_block_time_target(config);
        let max_block_time_target = ConsensusConfig::max_block_time_target(config);
        if (new_epoch_block_time_target < min_block_time_target) {
            new_epoch_block_time_target = min_block_time_target;
        };
        if (new_epoch_block_time_target > max_block_time_target) {
            new_epoch_block_time_target = max_block_time_target;
        };
        new_epoch_block_time_target
    }

    spec compute_next_block_time_target {
        pragma verify = false;
    }

    /// adjust_epoch try to advance to next epoch if current epoch ends.
    public fun adjust_epoch(account: &signer, block_number: u64, timestamp: u64, uncles: u64, parent_gas_used:u64): u128
    acquires Epoch, EpochData {
        CoreAddresses::assert_genesis_address(account);

        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        assert!(epoch_ref.max_uncles_per_block >= uncles, Errors::invalid_argument(EINVALID_UNCLES_COUNT));

        let epoch_data = borrow_global_mut<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        let (new_epoch, reward_per_block) = if (block_number < epoch_ref.end_block_number) {
            (false, epoch_ref.reward_per_block)
        } else if (block_number == epoch_ref.end_block_number) {
            //start a new epoch
            assert!(uncles == 0, Errors::invalid_argument(EINVALID_UNCLES_COUNT));
            // block time target unit is milli_seconds.
            let now_milli_seconds = timestamp;

            let config = ConsensusConfig::get_config();
            let last_epoch_time_target = epoch_ref.block_time_target;
            let new_epoch_block_time_target = compute_next_block_time_target(&config, last_epoch_time_target, epoch_ref.start_time, now_milli_seconds, epoch_ref.start_block_number, epoch_ref.end_block_number, epoch_data.uncles);
            let new_reward_per_block = ConsensusConfig::do_compute_reward_per_block(&config, new_epoch_block_time_target);

            //update epoch by adjust result or config, because ConsensusConfig may be updated.
            epoch_ref.number = epoch_ref.number + 1;
            epoch_ref.start_time = now_milli_seconds;
            epoch_ref.start_block_number = block_number;
            epoch_ref.end_block_number = block_number + ConsensusConfig::epoch_block_count(&config);
            epoch_ref.block_time_target = new_epoch_block_time_target;
            epoch_ref.reward_per_block = new_reward_per_block;
            epoch_ref.reward_per_uncle_percent = ConsensusConfig::base_reward_per_uncle_percent(&config);
            epoch_ref.block_difficulty_window = ConsensusConfig::base_block_difficulty_window(&config);
            epoch_ref.max_uncles_per_block = ConsensusConfig::base_max_uncles_per_block(&config);
            epoch_ref.strategy = ConsensusConfig::strategy(&config);

            epoch_data.uncles = 0;
            let last_epoch_total_gas = epoch_data.total_gas + (parent_gas_used as u128);
            adjust_gas_limit(&config, epoch_ref, last_epoch_time_target, new_epoch_block_time_target, last_epoch_total_gas);
            emit_epoch_event(epoch_ref, epoch_data.total_reward);
            (true, new_reward_per_block)
        } else {
            //This should never happened.
            abort EUNREACHABLE
        };
        let reward = reward_per_block +
                reward_per_block * (epoch_ref.reward_per_uncle_percent as u128) * (uncles as u128) / (HUNDRED as u128);
        update_epoch_data(epoch_data, new_epoch, reward, uncles, parent_gas_used);
        reward
    }

    spec adjust_epoch {
        pragma verify = false; //timeout
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<Epoch>(Signer::address_of(account));
        aborts_if global<Epoch>(Signer::address_of(account)).max_uncles_per_block < uncles;
        aborts_if exists<EpochData>(Signer::address_of(account));
        aborts_if block_number == global<Epoch>(Signer::address_of(account)).end_block_number && uncles != 0;
        // ...
    }

    fun adjust_gas_limit(config: &ConsensusConfig, epoch_ref: &mut Epoch, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas:u128) {
        let new_gas_limit = compute_gas_limit(config, last_epoch_time_target, new_epoch_time_target, epoch_ref.block_gas_limit, last_epoch_total_gas);
        if (Option::is_some(&new_gas_limit)) {
            epoch_ref.block_gas_limit = Option::destroy_some(new_gas_limit);
        }
    }

    spec adjust_gas_limit {
        pragma verify = false; //mul_div() timeout
    }

    /// Compute block's gas limit of next epoch.
    public fun compute_gas_limit(config: &ConsensusConfig, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128) : Option::Option<u64> {
        let epoch_block_count = (ConsensusConfig::epoch_block_count(config) as u128);
        let gas_limit_threshold = (last_epoch_total_gas >= Math::mul_div((last_epoch_block_gas_limit as u128) * epoch_block_count, (80 as u128), (HUNDRED as u128)));
        let new_gas_limit = Option::none<u64>();

        let min_block_time_target = ConsensusConfig::min_block_time_target(config);
        let max_block_time_target = ConsensusConfig::max_block_time_target(config);
        let base_block_gas_limit =  ConsensusConfig::base_block_gas_limit(config);
        if (last_epoch_time_target == new_epoch_time_target) {
            if (new_epoch_time_target == min_block_time_target && gas_limit_threshold) {
                let increase_gas_limit = in_or_decrease_gas_limit(last_epoch_block_gas_limit, 110, base_block_gas_limit);
                new_gas_limit = Option::some(increase_gas_limit);
            } else if (new_epoch_time_target == max_block_time_target && !gas_limit_threshold) {
                let decrease_gas_limit = in_or_decrease_gas_limit(last_epoch_block_gas_limit, 90, base_block_gas_limit);
                new_gas_limit = Option::some(decrease_gas_limit);
            }
        };

        new_gas_limit
    }

    spec compute_gas_limit {
        pragma verify = false; //mul_div() timeout
    }

    fun in_or_decrease_gas_limit(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64 {
        let tmp_gas_limit = Math::mul_div((last_epoch_block_gas_limit as u128), (percent as u128), (HUNDRED as u128));
        let new_gas_limit = if (tmp_gas_limit > (min_block_gas_limit  as u128)) {
            (tmp_gas_limit as u64)
        } else {
            min_block_gas_limit
        };

        new_gas_limit
    }

    spec in_or_decrease_gas_limit {
        include Math::MulDivAbortsIf{x: last_epoch_block_gas_limit, y: percent, z: HUNDRED};
        aborts_if Math::spec_mul_div() > MAX_U64;
    }

    fun update_epoch_data(epoch_data: &mut EpochData, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used:u64) {
        if (new_epoch) {
            epoch_data.total_reward = reward;
            epoch_data.uncles = uncles;
            epoch_data.total_gas = 0;
        } else {
            epoch_data.total_reward = epoch_data.total_reward + reward;
            epoch_data.uncles = epoch_data.uncles + uncles;
            epoch_data.total_gas = epoch_data.total_gas + (parent_gas_used as u128);
        }
    }

    spec update_epoch_data {
        aborts_if !new_epoch && epoch_data.total_reward + reward > MAX_U128;
        aborts_if !new_epoch && epoch_data.uncles + uncles > MAX_U64;
        aborts_if !new_epoch && epoch_data.total_gas + parent_gas_used > MAX_U128;
    }

    fun emit_epoch_event(epoch_ref: &mut Epoch, previous_epoch_total_reward: u128) {
        Event::emit_event(
            &mut epoch_ref.new_epoch_events,
            NewEpochEvent {
                number: epoch_ref.number,
                start_time: epoch_ref.start_time,
                start_block_number: epoch_ref.start_block_number,
                end_block_number: epoch_ref.end_block_number,
                block_time_target: epoch_ref.block_time_target,
                reward_per_block: epoch_ref.reward_per_block,
                previous_epoch_total_reward,
            },
        );
    }

    spec emit_epoch_event {
        aborts_if false;
    }

    /// Get start time of current epoch 
    public fun start_time(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.start_time
    }

    spec start_time {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get uncles number of current epoch
    public fun uncles(): u64 acquires EpochData {
        let epoch_data = borrow_global<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        epoch_data.uncles
    }

    spec uncles {
        aborts_if !exists<EpochData>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get total gas of current epoch
    public fun total_gas(): u128 acquires EpochData {
        let epoch_data = borrow_global<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        epoch_data.total_gas
    }

    spec total_gas {
        aborts_if !exists<EpochData>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get block's gas_limit of current epoch
    public fun block_gas_limit(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.block_gas_limit
    }

    spec block_gas_limit {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get start block's number of current epoch
    public fun start_block_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.start_block_number
    }

    spec start_block_number {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get end block's number of current epoch
    public fun end_block_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.end_block_number
    }

    spec end_block_number {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get current epoch number
    public fun number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.number
    }

    spec number {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get current block time target
    public fun block_time_target(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.block_time_target
    }

    spec block_time_target {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

}
}