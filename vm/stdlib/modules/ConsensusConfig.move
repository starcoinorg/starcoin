address 0x1 {
module ConsensusConfig {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::Errors;
    use 0x1::Timestamp;
    use 0x1::Math;
    use 0x1::Option;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    const THOUSAND: u64 = 1000;
    const THOUSAND_U128: u128 = 1000;
    const HUNDRED: u64 = 100;

    struct ConsensusConfig {
        uncle_rate_target: u64,
        base_block_time_target: u64,
        base_reward_per_block: u128,
        base_reward_per_uncle_percent: u64,
        epoch_block_count: u64,
        base_block_difficulty_window: u64,
        min_block_time_target: u64,
        max_block_time_target: u64,
        base_max_uncles_per_block: u64,
        base_block_gas_limit: u64,
        strategy: u8,
    }

    resource struct Epoch {
        epoch_number: u64,
        epoch_start_time: u64,
        start_number: u64,
        end_number: u64,
        block_time_target: u64,
        reward_per_block: u128,
        reward_per_uncle_percent: u64,
        block_difficulty_window: u64,
        max_uncles_per_block: u64,
        block_gas_limit: u64,
        strategy: u8,
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
        total_gas: u128,
    }

    const MAX_UNCLES_PER_BLOCK_IS_WRONG: u64 = 101;

    const UNCLES_IS_NOT_ZERO: u64 = 102;

    public fun initialize(
        account: &signer,
        uncle_rate_target: u64,
        epoch_block_count: u64,
        base_block_time_target: u64,
        base_block_difficulty_window: u64,
        base_reward_per_block: u128,
        base_reward_per_uncle_percent: u64,
        min_block_time_target: u64,
        max_block_time_target: u64,
        base_max_uncles_per_block: u64,
        base_block_gas_limit: u64,
        strategy: u8,
    ) {
        assert(Timestamp::is_genesis(), Errors::invalid_state(Errors::ENOT_GENESIS()));
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            Errors::requires_address(Errors::ENOT_GENESIS_ACCOUNT()),
        );

        Config::publish_new_config<Self::ConsensusConfig>(
            account,
            new_consensus_config(
                uncle_rate_target,
                base_block_time_target,
                base_reward_per_block,
                epoch_block_count,
                base_block_difficulty_window,
                base_reward_per_uncle_percent,
                min_block_time_target,
                max_block_time_target,
                base_max_uncles_per_block,
                base_block_gas_limit,
                strategy,
            ),
        );
        move_to<Epoch>(
            account,
            Epoch {
                epoch_number: 0,
                epoch_start_time: Timestamp::now_seconds(),
                start_number: 0,
                end_number: epoch_block_count,
                block_time_target: base_block_time_target,
                reward_per_block: base_reward_per_block,
                reward_per_uncle_percent: base_reward_per_uncle_percent,
                block_difficulty_window: base_block_difficulty_window,
                max_uncles_per_block: base_max_uncles_per_block,
                block_gas_limit: base_block_gas_limit,
                strategy: strategy,
                new_epoch_events: Event::new_event_handle<NewEpochEvent>(account),
            },
        );
        move_to<EpochData>(account, EpochData { uncles: 0, total_reward: 0, total_gas: 0 });
    }

    spec fun initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if uncle_rate_target == 0;
        aborts_if epoch_block_count == 0;
        aborts_if base_reward_per_block == 0;
        aborts_if base_block_time_target == 0;
        aborts_if base_block_difficulty_window == 0;
        aborts_if base_reward_per_uncle_percent == 0;
        aborts_if min_block_time_target == 0;
        aborts_if max_block_time_target < min_block_time_target;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());

        aborts_if exists<Epoch>(Signer::spec_address_of(account));
        aborts_if exists<EpochData>(Signer::spec_address_of(account));
        include Config::PublishNewConfigAbortsIf<ConsensusConfig>;
        include Config::PublishNewConfigEnsures<ConsensusConfig>;
    }

    public fun new_consensus_config(uncle_rate_target: u64,
                                    base_block_time_target: u64,
                                    base_reward_per_block: u128,
                                    base_reward_per_uncle_percent: u64,
                                    epoch_block_count: u64,
                                    base_block_difficulty_window: u64,
                                    min_block_time_target: u64,
                                    max_block_time_target: u64,
                                    base_max_uncles_per_block: u64,
                                    base_block_gas_limit: u64,
                                    strategy: u8,): ConsensusConfig {
        assert(uncle_rate_target > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(base_block_time_target > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(base_reward_per_block > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(epoch_block_count > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(base_block_difficulty_window > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(base_reward_per_uncle_percent > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(min_block_time_target > 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(max_block_time_target >= min_block_time_target, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(base_max_uncles_per_block >= 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(base_block_gas_limit >= 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));
        assert(strategy >= 0, Errors::invalid_argument(Errors::EINVALID_ARGUMENT()));

        ConsensusConfig {
            uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            epoch_block_count,
            base_block_difficulty_window,
            base_reward_per_uncle_percent,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy,
        }
    }

    spec fun new_consensus_config {
        aborts_if uncle_rate_target == 0;
        aborts_if epoch_block_count == 0;
        aborts_if base_reward_per_block == 0;
        aborts_if base_block_time_target == 0;
        aborts_if base_block_difficulty_window == 0;
        aborts_if base_reward_per_uncle_percent == 0;
        aborts_if min_block_time_target == 0;
        aborts_if max_block_time_target < min_block_time_target;
    }

    public fun get_config(): ConsensusConfig {
        Config::get_by_address<ConsensusConfig>(CoreAddresses::GENESIS_ADDRESS())
    }

    spec fun get_config {
        aborts_if !exists<Config::Config<ConsensusConfig>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    spec define spec_get_config(): ConsensusConfig {
        global<Config::Config<ConsensusConfig>>(CoreAddresses::SPEC_GENESIS_ADDRESS()).payload
    }

    public fun uncle_rate_target(config: &ConsensusConfig): u64 {
        config.uncle_rate_target
    }
    
    public fun base_block_time_target(config: &ConsensusConfig): u64 {
        config.base_block_time_target
    }

    public fun base_reword_per_block(config: &ConsensusConfig): u128 {
        config.base_reward_per_block
    }
    
    public fun epoch_block_count(config: &ConsensusConfig): u64 {
        config.epoch_block_count
    }

    public fun base_block_difficulty_window(config: &ConsensusConfig): u64 {
        config.base_block_difficulty_window
    }

    public fun base_reward_per_uncle_percent(config: &ConsensusConfig): u64 {
        config.base_reward_per_uncle_percent
    }

    public fun min_block_time_target(config: &ConsensusConfig): u64 {
        config.min_block_time_target
    }

    public fun max_block_time_target(config: &ConsensusConfig): u64 {
        config.max_block_time_target
    }

    public fun base_max_uncles_per_block(config: &ConsensusConfig): u64 {
        config.base_max_uncles_per_block
    }

    public fun base_block_gas_limit(config: &ConsensusConfig): u64 {
        config.base_block_gas_limit
    }

    public fun strategy(config: &ConsensusConfig): u8 {
        config.strategy
    }

    public fun compute_reward_per_block(new_epoch_block_time_target: u64): u128 {
        let config = get_config();
        do_compute_reward_per_block(&config, new_epoch_block_time_target)
    }

    spec fun compute_reward_per_block {
        aborts_if !exists<Config::Config<ConsensusConfig>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if spec_get_config().base_reward_per_block * new_epoch_block_time_target > MAX_U128;
        aborts_if spec_get_config().base_reward_per_block * new_epoch_block_time_target * THOUSAND_U128 > MAX_U128;
        aborts_if spec_get_config().base_block_time_target == 0;
    }

    fun do_compute_reward_per_block(config: &ConsensusConfig, new_epoch_block_time_target: u64): u128 {
        config.base_reward_per_block *
                (new_epoch_block_time_target as u128) * THOUSAND_U128 /
                (config.base_block_time_target as u128) / THOUSAND_U128
    }

    spec fun do_compute_reward_per_block {
        aborts_if config.base_reward_per_block * new_epoch_block_time_target > MAX_U128;
        aborts_if config.base_reward_per_block * new_epoch_block_time_target * THOUSAND_U128 > MAX_U128;
        aborts_if config.base_block_time_target == 0;
    }

    public fun adjust_epoch(account: &signer, block_number: u64, now: u64, uncles: u64, parent_gas_used:u64): u128
    acquires Epoch, EpochData {
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            Errors::requires_address(Errors::ENOT_GENESIS_ACCOUNT()),
        );

        let epoch_ref = borrow_global_mut<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        assert(epoch_ref.max_uncles_per_block >= uncles, Errors::invalid_argument(MAX_UNCLES_PER_BLOCK_IS_WRONG));

        let epoch_data = borrow_global_mut<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        let (new_epoch, reward_per_block) = if (block_number < epoch_ref.end_number) {
            (false, epoch_ref.reward_per_block)
        } else if (block_number == epoch_ref.end_number) {
            //start a new epoch
            assert(uncles == 0, Errors::invalid_argument(UNCLES_IS_NOT_ZERO));
            let config = get_config();
            let last_epoch_time_target = epoch_ref.block_time_target;
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
            let new_reward_per_block = do_compute_reward_per_block(&config, new_epoch_block_time_target);

            //update epoch by adjust result or config, because ConsensusConfig may be updated.
            epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
            epoch_ref.epoch_start_time = now;
            epoch_ref.start_number = block_number;
            epoch_ref.end_number = block_number + config.epoch_block_count;
            epoch_ref.block_time_target = new_epoch_block_time_target;
            epoch_ref.reward_per_block = new_reward_per_block;
            epoch_ref.reward_per_uncle_percent = config.base_reward_per_uncle_percent;
            epoch_ref.block_difficulty_window = config.base_block_difficulty_window;
            epoch_ref.max_uncles_per_block = config.base_max_uncles_per_block;
            epoch_ref.strategy = config.strategy;

            epoch_data.uncles = 0;
            let last_epoch_total_gas = epoch_data.total_gas + (parent_gas_used as u128);
            adjust_gas_limit(&config, epoch_ref, last_epoch_time_target, new_epoch_block_time_target, last_epoch_total_gas);
            emit_epoch_event(epoch_ref, epoch_data.total_reward);
            (true, new_reward_per_block)
        } else {
            //This should never happened.
            abort Errors::EUNREACHABLE()
        };
        let reward = reward_per_block +
            reward_per_block * (epoch_ref.reward_per_uncle_percent as u128) * (uncles as u128) / (HUNDRED as u128);
        update_epoch_data(epoch_data, new_epoch, reward, uncles, parent_gas_used);
        reward
    }

    spec fun adjust_epoch {
        pragma verify = false; //timeout
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<Epoch>(Signer::spec_address_of(account));
        aborts_if global<Epoch>(Signer::spec_address_of(account)).max_uncles_per_block < uncles;
        aborts_if exists<EpochData>(Signer::spec_address_of(account));
        aborts_if block_number == global<Epoch>(Signer::spec_address_of(account)).end_number && uncles != 0;
        // ...
    }

    fun adjust_gas_limit(config: &ConsensusConfig, epoch_ref: &mut Epoch, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas:u128) {
        let new_gas_limit = compute_gas_limit(config, last_epoch_time_target, new_epoch_time_target, epoch_ref.block_gas_limit, last_epoch_total_gas);
        if (Option::is_some(&new_gas_limit)) {
            epoch_ref.block_gas_limit = Option::destroy_some(new_gas_limit);
        }
    }

    spec fun adjust_gas_limit {
        pragma verify = false; //mul_div() timeout
    }

    public fun compute_gas_limit(config: &ConsensusConfig, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128) : Option::Option<u64> {
        let gas_limit_threshold = (last_epoch_total_gas >= Math::mul_div((last_epoch_block_gas_limit as u128) * (config.epoch_block_count as u128), (80 as u128), (HUNDRED as u128)));
        let new_gas_limit = Option::none<u64>();
        if (last_epoch_time_target == new_epoch_time_target) {
            if (new_epoch_time_target == config.min_block_time_target && gas_limit_threshold) {
                let increase_gas_limit = in_or_decrease_gas_limit(last_epoch_block_gas_limit, 110, config.base_block_gas_limit);
                new_gas_limit = Option::some(increase_gas_limit);
            } else if (new_epoch_time_target == config.max_block_time_target && !gas_limit_threshold) {
                let decrease_gas_limit = in_or_decrease_gas_limit(last_epoch_block_gas_limit, 90, config.base_block_gas_limit);
                new_gas_limit = Option::some(decrease_gas_limit);
            }
        };

        new_gas_limit
    }

    spec fun compute_gas_limit {
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

    spec fun in_or_decrease_gas_limit {
        pragma verify = false; //mul_div() timeout
    }

    fun update_epoch_data(epoch_data: &mut EpochData, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used:u64) {
        if (new_epoch) {
            epoch_data.total_reward = reward;
            epoch_data.uncles = uncles;
            epoch_data.total_reward = 0;
        } else {
            epoch_data.total_reward = epoch_data.total_reward + reward;
            epoch_data.uncles = epoch_data.uncles + uncles;
            epoch_data.total_gas = epoch_data.total_gas + (parent_gas_used as u128);
        }
    }

    spec fun update_epoch_data {
        aborts_if !new_epoch && epoch_data.total_reward + reward > MAX_U128;
        aborts_if !new_epoch && epoch_data.uncles + uncles > MAX_U64;
        aborts_if !new_epoch && epoch_data.total_gas + parent_gas_used > MAX_U128;
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

    spec fun emit_epoch_event {
        aborts_if false;
    }

    public fun epoch_start_time(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.epoch_start_time
    }

    spec fun epoch_start_time {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun uncles(): u64 acquires EpochData {
        let epoch_data = borrow_global<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        epoch_data.uncles
    }

    spec fun uncles {
        aborts_if !exists<EpochData>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun epoch_total_gas(): u128 acquires EpochData {
        let epoch_data = borrow_global<EpochData>(CoreAddresses::GENESIS_ADDRESS());
        epoch_data.total_gas
    }

    spec fun epoch_total_gas {
        aborts_if !exists<EpochData>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun epoch_block_gas_limit(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.block_gas_limit
    }

    spec fun epoch_block_gas_limit {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun epoch_start_block_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.start_number
    }

    spec fun epoch_start_block_number {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun epoch_end_block_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.end_number
    }

    spec fun epoch_end_block_number {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun epoch_number(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.epoch_number
    }

    spec fun epoch_number {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun block_time_target(): u64 acquires Epoch {
        let epoch_ref = borrow_global<Epoch>(CoreAddresses::GENESIS_ADDRESS());
        epoch_ref.block_time_target
    }

    spec fun block_time_target {
        aborts_if !exists<Epoch>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

}
}