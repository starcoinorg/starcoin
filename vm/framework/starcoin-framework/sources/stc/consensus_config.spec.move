spec starcoin_framework::consensus_config {
    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
    }

    spec initialize {
        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if uncle_rate_target == 0;
        aborts_if epoch_block_count == 0;
        aborts_if base_reward_per_block == 0;
        aborts_if base_block_time_target == 0;
        aborts_if base_block_difficulty_window == 0;
        aborts_if min_block_time_target == 0;
        aborts_if max_block_time_target < min_block_time_target;

        include on_chain_config::PublishNewConfigAbortsIf<ConsensusConfig>;
        include on_chain_config::PublishNewConfigEnsures<ConsensusConfig>;
    }


    spec new_consensus_config {
        aborts_if uncle_rate_target == 0;
        aborts_if epoch_block_count == 0;
        aborts_if base_reward_per_block == 0;
        aborts_if base_block_time_target == 0;
        aborts_if base_block_difficulty_window == 0;
        aborts_if min_block_time_target == 0;
        aborts_if max_block_time_target < min_block_time_target;
    }

    spec get_config {
        aborts_if !exists<on_chain_config::Config<ConsensusConfig>>(system_addresses::get_starcoin_framework());
    }

    spec fun spec_get_config(): ConsensusConfig {
        global<on_chain_config::Config<ConsensusConfig>>(system_addresses::get_starcoin_framework()).payload
    }

    spec compute_reward_per_block {
        aborts_if !exists<on_chain_config::Config<ConsensusConfig>>(system_addresses::get_starcoin_framework());
        include math128::MulDivAbortsIf {
            a: spec_get_config().base_reward_per_block,
            b: new_epoch_block_time_target,
            c: spec_get_config().base_block_time_target
        };
    }

    spec do_compute_reward_per_block {
        include math128::MulDivAbortsIf {
            a: config.base_reward_per_block,
            b: new_epoch_block_time_target,
            c: config.base_block_time_target
        };
    }
}
