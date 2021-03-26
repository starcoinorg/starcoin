//! account: alice

//! sender: alice
script {
use 0x1::Epoch;
    //ENOT_GENESIS_ACCOUNT
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}

// check: "Keep(ABORTED { code: 2818"

//! new-transaction
//! sender: genesis
script {
    use 0x1::Epoch;
    //block_number < epoch_ref.end_block_number, do nothing
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
    use 0x1::Epoch;
    use 0x1::ConsensusConfig;
    //EINVALID_UNCLES_COUNT
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let config = ConsensusConfig::get_config();
        let max_uncles_per_block = ConsensusConfig::base_max_uncles_per_block(&config);
        let uncles = max_uncles_per_block + 1;
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}
// check: "Keep(ABORTED { code: 25863"

//! new-transaction
//! sender: genesis
script {
    use 0x1::Epoch;
    use 0x1::ConsensusConfig;
    //EUNREACHABLE, block_number > epoch_ref.end_block_number
    fun adjust_epoch(genesis_account: signer) {
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let config = ConsensusConfig::get_config();
        let block_number = 1 + ConsensusConfig::epoch_block_count(&config);
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}
// check: "Keep(ABORTED { code: 19"

//! new-transaction
//! sender: genesis
script {
    use 0x1::Epoch;
    use 0x1::ConsensusConfig;
    //EINVALID_UNCLES_COUNT. If block_number == epoch_ref.end_block_number, uncles should be 0
    fun adjust_epoch(genesis_account: signer) {
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let config = ConsensusConfig::get_config();
        let block_number = ConsensusConfig::epoch_block_count(&config);
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}
// check: "Keep(ABORTED { code: 25863"

//! new-transaction
//! sender: genesis
script {
    use 0x1::Epoch;
    use 0x1::ConsensusConfig;
    //block_number == epoch_ref.end_block_number
    fun adjust_epoch(genesis_account: signer) {
        let block_time_milliseonds = 1000;
        let uncles = 0;
        let config = ConsensusConfig::get_config();
        let block_number = ConsensusConfig::epoch_block_count(&config);
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}
// check: EXECUTED
