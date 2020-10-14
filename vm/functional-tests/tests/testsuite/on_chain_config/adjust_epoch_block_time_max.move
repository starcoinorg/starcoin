// test adjust epoch full uncle.

//! sender: genesis
script {
use 0x1::ConsensusConfig;
use 0x1::Debug;

    fun main(genesis_account: &signer) {
        let block_number = 1;
        let block_time = 1;
        let times = 0;
        let config = ConsensusConfig::get_config();
        let base_block_time_target = ConsensusConfig::base_block_time_target(&config);
        let max_block_time_target = ConsensusConfig::max_block_time_target(&config);
        while (ConsensusConfig::epoch_number() < 10) {
            let uncles = 1;
            if (block_number == ConsensusConfig::epoch_end_block_number()) {
                uncles = 0;
                Debug::print(&ConsensusConfig::block_time_target());
            };
            let _reward = ConsensusConfig::adjust_epoch(genesis_account, block_number, block_time, uncles, 0);

            let block_time_target = ConsensusConfig::block_time_target();
            assert(block_time_target >= base_block_time_target, 102);
            assert(block_time_target <= max_block_time_target, 103);
            times = times + 1;
            block_number = block_number + 1;
            block_time = block_time + block_time_target;
        };
    }
}

// check: EXECUTED
