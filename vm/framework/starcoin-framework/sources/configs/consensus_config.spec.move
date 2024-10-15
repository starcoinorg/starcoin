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
        include MulDivAbortsIf {
            x: spec_get_config().base_reward_per_block,
            y: new_epoch_block_time_target,
            z: spec_get_config().base_block_time_target
        };
    }

    spec do_compute_reward_per_block {
        include MulDivAbortsIf {
            x: config.base_reward_per_block,
            y: new_epoch_block_time_target,
            z: config.base_block_time_target
        };
    }

    spec schema MulDivAbortsIf {
        x: u128;
        y: u128;
        z: u128;
        aborts_if y != z && x > z && z == 0;
        aborts_if y != z && x > z && z != 0 && x / z * y > MAX_U128;
        aborts_if y != z && x <= z && z == 0;
        //a * b overflow
        aborts_if y != z && x <= z && x / z * (x % z) > MAX_U128;
        //a * b * z overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z > MAX_U128;
        //a * d overflow
        aborts_if y != z && x <= z && x / z * (y % z) > MAX_U128;
        //a * b * z + a * d overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z + x / z * (y % z) > MAX_U128;
        //b * c overflow
        aborts_if y != z && x <= z && x % z * (y / z) > MAX_U128;
        //b * d overflow
        aborts_if y != z && x <= z && x % z * (y % z) > MAX_U128;
        //b * d / z overflow
        aborts_if y != z && x <= z && x % z * (y % z) / z > MAX_U128;
        //a * b * z + a * d + b * c overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z + x / z * (y % z) + x % z * (y / z) > MAX_U128;
        //a * b * z + a * d + b * c + b * d / z overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z + x / z * (y % z) + x % z * (y / z) + x % z * (y % z) / z > MAX_U128;
    }
}
