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
        let max_transaction_per_block: u64 = 3000;

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

        let total_uncles = 0;
        let now_milli_seconds = epoch_block_count * base_block_time_target;
        let last_block_time_target = base_block_time_target;
        assert!(
            epoch::compute_next_block_time_target(
                &config,
                last_block_time_target,
                0,
                now_milli_seconds,
                total_uncles,
                0,
            ) < base_block_time_target,
            101
        );

        let total_uncles = epoch_block_count * uncle_rate_target / 1000;
        let new_block_time_target = epoch::compute_next_block_time_target(
            &config,
            last_block_time_target,
            0,
            now_milli_seconds,
            total_uncles,
            0,
        );
        assert!(
            new_block_time_target >= base_block_time_target - 1 || new_block_time_target <= base_block_time_target + 1,
            102
        );

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
        assert!(new_block_time_target == last_block_time_target / 2, 103);

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
        assert!(new_block_time_target == last_block_time_target * 2, 104);

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
        assert!(new_block_time_target == max_block_time_target, 105);

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
        assert!(new_block_time_target == min_block_time_target, 105);
    }
}