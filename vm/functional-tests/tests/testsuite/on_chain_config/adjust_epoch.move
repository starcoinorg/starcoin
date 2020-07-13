//! sender: genesis
script {
use 0x1::Consensus;

fun main(genesis_account: &signer) {
    let times = 0;
    let block_height = 1;
    let block_time = 1;

    while (times < 100) {
        Consensus::adjust_epoch(genesis_account, block_height, block_time, 0);

        if (block_height == 13) {
            assert(Consensus::start_number() == 13, 10000);
            assert(Consensus::epoch_start_time() == 121, 10001);
            assert(Consensus::end_number() == 49, 10002);
            assert(Consensus::time_target() == 10, 10003);
        };

        if (block_height == 50) {
            assert(Consensus::start_number() == 50, 10004);
            assert(Consensus::epoch_start_time() == 491, 10005);
            assert(Consensus::end_number() == 61, 10006);
            assert(Consensus::time_target() == 10, 10007);
        };

        block_height = block_height + 1;

        block_time = block_time + 10;
        times = times + 1;
    };

    assert(Consensus::start_number() > 0, 1000);
}
}
