//! account: alice, 100000000
//! new-transaction
//! sender: alice
module TestAdjustEpoch {
use 0x1::Consensus;

public fun test(genesis_account: &signer, block_number: u64, block_time: u64) {
    let times = 0;
    //just test not abort.
    while (times < 1000) {
        let _reward = Consensus::adjust_epoch(genesis_account, block_number + times, block_time + times, 0);
        times = times + 1;
    };
}
}

// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x1::Account;
use 0x1::CoreAddresses;
use 0x1::STC::{STC};

fun main(association_account: &signer) {
    Account::pay_from<STC>(association_account, CoreAddresses::GENESIS_ADDRESS(), 400000000000000);
}
}

// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
use {{alice}}::TestAdjustEpoch;
use 0x1::Consensus;

fun main(genesis_account: &signer) {
    let times = 0;
    let block_number = 1;
    let block_time = 1;

    while (times < 3) {
        TestAdjustEpoch::test(genesis_account, block_number, block_time);

        block_number = block_number + (times + 1) * 10;
        block_time = block_time + (times + 1) * 10;
        times = times + 1;
    };

    assert(Consensus::start_number() > 0, 1000);
    assert(Consensus::epoch_number() > 0, 1001);
}
}
