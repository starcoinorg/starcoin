
// test adjust epoch zero uncle.
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
    let min_block_time_target = ConsensusConfig::min_block_time_target(&config);
    while (ConsensusConfig::epoch_number() < 10) {
        if (block_number == ConsensusConfig::epoch_end_block_number()) {
            Debug::print(&ConsensusConfig::block_time_target());
        };
        let _reward = ConsensusConfig::adjust_epoch(genesis_account, block_number, block_time, 0);
        let block_time_target = ConsensusConfig::block_time_target();
        assert(block_time_target >= min_block_time_target, 100);
        assert(block_time_target <= base_block_time_target, 101);
        times = times + 1;
        block_number = block_number + 1;
        block_time = block_time + block_time_target;
    };
}
}


// check: EXECUTED

