script {
    use 0x1::ConsensusConfig;

    fun main() {
        let config = ConsensusConfig::get_config();
        assert(ConsensusConfig::uncle_rate_target(&config) == 80, 8100);
        //assert(ConsensusConfig::epoch_time_target() == 1209600, 8101);
    //assert(ConsensusConfig::reward_half_time_target() == 126144000, 8102);

    }
}

//! new-transaction
script {
    use 0x1::ConsensusConfig;

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

        assert(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target * 2) == base_reward_per_block * 2, 101);
        assert(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target / 2) == base_reward_per_block / 2, 102);
        assert(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target / 5) == base_reward_per_block / 5, 103);
        assert(ConsensusConfig::do_compute_reward_per_block(&config, base_block_time_target / base_block_time_target) == base_reward_per_block / (base_block_time_target as u128), 104);
    }
}