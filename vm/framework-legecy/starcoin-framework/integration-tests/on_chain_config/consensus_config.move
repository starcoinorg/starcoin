//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

    // test do_compute_reward_per_block()
    fun compute_reward_per_block() {
        let uncle_rate_target = 80;
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

        assert!(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target * 2) == base_reward_per_block * 2, 101);
        assert!(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target / 2) == base_reward_per_block / 2, 102);
        assert!(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target / 5) == base_reward_per_block / 5, 103);
        assert!(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target / base_block_time_target) == base_reward_per_block / (base_block_time_target as u128), 104);
    }
}

//# run --signers alice
// test compute_reward_per_block
script {
    use StarcoinFramework::ConsensusConfig;

    fun compute_reward_per_block() {
        let block_time_target = 10000; // equal to default block_time_target
        let default_reward_per_block = 10000000000; // should be consistent with genesis config
        let reward_per_block = ConsensusConfig::compute_reward_per_block(block_time_target);
        assert!(reward_per_block == default_reward_per_block, 102);
    }
}

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}
// check: "Keep(ABORTED { code: 4615"

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}
// check: "Keep(ABORTED { code: 4615"

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}
// check: "Keep(ABORTED { code: 4615"

//# run --signers alice
script {
    use StarcoinFramework::ConsensusConfig;

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

        ConsensusConfig::new_consensus_config(
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

    }
}
// check: "Keep(ABORTED { code: 4615"
