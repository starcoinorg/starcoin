
// test adjust epoch zero uncle.
//! sender: genesis
script {
use 0x1::Consensus;
use 0x1::Debug;

fun main(genesis_account: &signer) {
    let block_number = 1;
    let block_time = 1;
    let times = 0;
    let init_block_time_target = Consensus::init_block_time_target();
    let min_block_time_target = Consensus::min_block_time_target();
    while (Consensus::epoch_number() < 10) {
        if (block_number == Consensus::epoch_end_block_number()) {
            Debug::print(&Consensus::block_time_target());
        };
        let _reward = Consensus::adjust_epoch(genesis_account, block_number, block_time, 0);
        let block_time_target = Consensus::block_time_target();
        assert(block_time_target >= min_block_time_target, 100);
        assert(block_time_target <= init_block_time_target, 101);
        times = times + 1;
        block_number = block_number + 1;
        block_time = block_time + block_time_target;
    };
}
}


// check: EXECUTED

