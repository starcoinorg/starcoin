//# init -n dev

//# faucet --addr Genesis

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::Epoch;
    //ENOT_GENESIS_ACCOUNT
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}

// check: "Keep(ABORTED { code: 2818"


//# run --signers Genesis
script {
    use StarcoinFramework::Epoch;
    //block_number < epoch_ref.end_block_number, do nothing
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}

// check: EXECUTED


//# run --signers Genesis
script {
    use StarcoinFramework::Epoch;
    use StarcoinFramework::ConsensusConfig;
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


//# run --signers Genesis
script {
    use StarcoinFramework::Epoch;
    use StarcoinFramework::ConsensusConfig;
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


//# run --signers Genesis
script {
    use StarcoinFramework::Epoch;
    use StarcoinFramework::ConsensusConfig;
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


//# run --signers Genesis
script {
    use StarcoinFramework::Epoch;
    use StarcoinFramework::ConsensusConfig;
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
