
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

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    // test do_compute_reward_per_block()
    fun compute_reward_per_block() {
        let uncle_rate_target = 1;
        let base_reward_per_block = 10000;
        let base_block_time_target = 1000;
        let base_reward_per_uncle_percent = 10;
        let epoch_block_count = 96;
        let base_block_difficulty_window = 48;
        let min_block_time_target = 1000;
        let max_block_time_target = 2000;
        let base_max_uncles_per_block = 16;
        let base_block_gas_limit = 10000;
        let strategy = 1;

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
            0,
            0,
            0
        );

        assert!(
            consensus_config::do_compute_reward_per_block(
                &config,
                base_block_time_target * 2
            ) == base_reward_per_block * 2,
            101
        );
        assert!(
            consensus_config::do_compute_reward_per_block(
                &config,
                base_block_time_target / 2
            ) == base_reward_per_block / 2,
            102
        );
        assert!(
            consensus_config::do_compute_reward_per_block(
                &config,
                base_block_time_target / 5
            ) == base_reward_per_block / 5,
            103
        );
        assert!(
            consensus_config::do_compute_reward_per_block(
                &config,
                base_block_time_target / base_block_time_target
            ) == base_reward_per_block / (base_block_time_target as u128),
            104
        );
    }
}

//# run --signers alice
script {
    use starcoin_std::debug;
    use starcoin_framework::consensus_config;

    fun compute_reward_per_block() {
        let block_time_target = 1000; // equal to default block_time_target
        let default_reward_per_block = 10000000000; // should be consistent with genesis config
        let reward_per_block = consensus_config::compute_reward_per_block(block_time_target);
        debug::print(&b"consensus_config::compute_reward_per_block");
        assert!(reward_per_block == default_reward_per_block, 102);
    }
}

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 0; // should large than 0
        let base_reward_per_block = 10000;
        let base_block_time_target = 10;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 24;
        let min_block_time_target = 5;
        let max_block_time_target = 60;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 0; // should large than 0
        let base_block_time_target = 10;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 24;
        let min_block_time_target = 5;
        let max_block_time_target = 60;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 10000;
        let base_block_time_target = 0; // should large than 0
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 24;
        let min_block_time_target = 5;
        let max_block_time_target = 60;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 10000;
        let base_block_time_target = 10;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 0; // should large than 0
        let base_block_difficulty_window = 24;
        let min_block_time_target = 5;
        let max_block_time_target = 60;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 10000;
        let base_block_time_target = 10;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 0;  // should large than 0
        let min_block_time_target = 5;
        let max_block_time_target = 60;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 10000;
        let base_block_time_target = 10;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 24;
        let min_block_time_target = 0; // should large than 0
        let max_block_time_target = 60;
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"

//# run --signers alice
script {
    use starcoin_framework::consensus_config;

    fun incorrect_uncle_rate_target() {
        let uncle_rate_target = 80;
        let base_reward_per_block = 10000;
        let base_block_time_target = 10;
        let base_reward_per_uncle_percent = 0;
        let epoch_block_count = 240;
        let base_block_difficulty_window = 24;
        let min_block_time_target = 5;
        let max_block_time_target = 4; //max_block_time_target should large than min_block_time_target
        let base_max_uncles_per_block = 2;
        let base_block_gas_limit = 10000;
        let strategy = 1;

        consensus_config::new_consensus_config(
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
            strategy, 0, 0, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"
