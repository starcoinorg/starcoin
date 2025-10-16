/// We refer to the default configuration of genesis as follows
///
/// static G_UNCLE_RATE_TARGET: u64 = 1;
/// static G_DEFAULT_BASE_BLOCK_TIME_TARGET: u64 = 1000;
/// static G_DEFAULT_BASE_BLOCK_DIFF_WINDOW: u64 = 48;
/// static G_BASE_REWARD_PER_UNCLE_PERCENT: u64 = 10;
/// static G_MIN_BLOCK_TIME_TARGET: u64 = 1000;
/// static G_MAX_BLOCK_TIME_TARGET: u64 = 2000;
/// pub static G_BASE_MAX_UNCLES_PER_BLOCK: u64 = 16;
///


//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use starcoin_framework::epoch;
    use starcoin_framework::consensus_config;

    fun compute_next_block_time_target() {
        let uncle_rate_target = 1;
        let base_reward_per_block = 10000;
        let base_block_time_target = 1000;
        let base_reward_per_uncle_percent = 10;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 48;
        let min_block_time_target = 1000;
        let max_block_time_target = 2000;
        let base_max_uncles_per_block = 16;
        let base_block_gas_limit = 10000;
        let strategy = 1;
        let pruning_depth: u64 = 185798;  // DAG pruning parameters
        let pruning_finality: u64 = 86400;
        let max_transaction_per_block: u64 = 700;

        let config = consensus_config::new_consensus_config(
            uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            base_reward_per_uncle_percent,
            epoch_block_count,
            base_block_difficulty_window,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy,
            max_transaction_per_block,
            pruning_depth,
            pruning_finality,
        );

        // Test case 1: No uncles, expected time = average_time * 2/3
        // expected_blue_count = 240 * (16-1) / 1 = 3600
        // actual blue_blocks = 0, so blue_blocks < expected_blue_count
        // average_time = 240000 / 240 = 1000
        // new_target = 1000 * 2/3 = 666
        // But min_block_time_target = 1000, so result is 1000
        let total_uncles = 0;
        let now_milli_seconds = epoch_block_count * base_block_time_target;
        let last_block_time_target = base_block_time_target;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(new_block_time_target == min_block_time_target, 101);

        // Test case 2: Very few uncles (close to 0)
        // total_uncles = 240 * 1 / 1000 = 0 (rounds down)
        // Same as test case 1, result bounded by min
        let total_uncles = epoch_block_count * uncle_rate_target / 1000;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(new_block_time_target == min_block_time_target, 102);

        // Test case 3: Fast block production (half the time)
        // average_time = 120000 / 240 = 500
        // expected_blue_count = 3600, actual = 0
        // new_target = 500 * 2/3 = 333
        // Stability constraint: can't go below 1000 * 2/3 = 666
        // But min bound = 1000, so result is 1000
        let total_uncles = epoch_block_count * uncle_rate_target / 1000;
        let now_milli_seconds = epoch_block_count * base_block_time_target / 2;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(new_block_time_target == min_block_time_target, 103);

        // Test case 4: Many uncles (479 uncles)
        // expected_blue_count = 3600, actual = 479
        // Since 479 < 3600, still multiply by 2/3
        // average_time = 240000 / (240 + 479) = 333
        // new_target = 333 * 2/3 = 222
        // Stability constraint: can't go below 1000 * 2/3 = 666
        // Min bound = 1000, so result is 1000
        let total_uncles = epoch_block_count * 2 - 1;
        let now_milli_seconds = epoch_block_count * base_block_time_target;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(new_block_time_target == min_block_time_target, 104);

        // Test case 5: Last epoch near max, with many uncles
        // last_block_time_target = 1999
        // average_time = 479760 / 719 = 667
        // expected_blue_count = 3600, actual = 479 < 3600
        // new_target = 667 * 2/3 = 444
        // But stability constraint: can't go below 1999 * 2/3 = 1332
        let last_block_time_target = max_block_time_target - 1;
        let total_uncles = epoch_block_count * 2 - 1;
        let now_milli_seconds = epoch_block_count * last_block_time_target;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(new_block_time_target == 1332 || new_block_time_target == 1333, 105);

        // Test case 6: Already at minimum, no uncles
        // last_block_time_target = 1000 (min)
        // average_time = 240000 / 240 = 1000
        // expected_blue_count = 3600, actual = 0 < 3600
        // new_target = 1000 * 2/3 = 666
        // Min bound applies, stays at min
        let last_block_time_target = min_block_time_target;
        let total_uncles = 0;
        let now_milli_seconds = epoch_block_count * min_block_time_target;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(new_block_time_target == min_block_time_target, 106);
    }
}
