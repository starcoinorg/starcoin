//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;
    use StarcoinFramework::Epoch;

    fun compute_next_block_time_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 10000;
        let base_block_time_target = 10000;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 24;
        let min_block_time_target = 2000;
        let max_block_time_target = 60000;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        let config = ConsensusConfig::new_consensus_config(
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
            strategy);

        let total_uncles = 0;
        let now_milli_seconds = epoch_block_count*base_block_time_target;
        let last_block_time_target = base_block_time_target;
        assert!(Epoch::compute_next_block_time_target(&config, last_block_time_target, 0, now_milli_seconds,  0, epoch_block_count, total_uncles) < base_block_time_target, 101);

        let total_uncles = epoch_block_count * uncle_rate_target /1000;
        let new_block_time_target = Epoch::compute_next_block_time_target(&config, last_block_time_target, 0, now_milli_seconds,  0, epoch_block_count, total_uncles);
        assert!(new_block_time_target >= base_block_time_target -1 || new_block_time_target <= base_block_time_target +1, 102);

        let total_uncles = epoch_block_count * uncle_rate_target /1000;
        let now_milli_seconds = epoch_block_count*base_block_time_target/2;
        let new_block_time_target = Epoch::compute_next_block_time_target(&config, last_block_time_target, 0, now_milli_seconds,  0, epoch_block_count, total_uncles);
        assert!(new_block_time_target == last_block_time_target/2, 103);

        let total_uncles = epoch_block_count*2-1;
        let now_milli_seconds = epoch_block_count*base_block_time_target;
        let new_block_time_target = Epoch::compute_next_block_time_target(&config, last_block_time_target, 0, now_milli_seconds,  0, epoch_block_count, total_uncles);
        assert!(new_block_time_target == last_block_time_target*2, 104);

        let last_block_time_target = max_block_time_target - 1;
        let total_uncles = epoch_block_count*2-1;
        let now_milli_seconds = epoch_block_count*last_block_time_target;
        let new_block_time_target = Epoch::compute_next_block_time_target(&config, last_block_time_target , 0, now_milli_seconds,  0, epoch_block_count, total_uncles);
        assert!(new_block_time_target == max_block_time_target, 105);

        let last_block_time_target = min_block_time_target;
        let total_uncles = 0;
        let now_milli_seconds = epoch_block_count*min_block_time_target;
        let new_block_time_target = Epoch::compute_next_block_time_target(&config, last_block_time_target , 0, now_milli_seconds,  0, epoch_block_count, total_uncles);
        assert!(new_block_time_target == min_block_time_target, 105);
    }
}