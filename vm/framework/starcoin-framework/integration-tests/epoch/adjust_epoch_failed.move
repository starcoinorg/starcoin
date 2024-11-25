//# init -n dev

//# faucet --addr Genesis

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::epoch;

    // ENOT_GENESIS_ACCOUNT
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let _reward = epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}

// check: "Keep(ABORTED { code: 2818"


//# run --signers Genesis
script {
    use starcoin_framework::epoch;

    //block_number < epoch_ref.end_block_number, do nothing
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let _reward = epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}

// check: EXECUTED


//# run --signers Genesis
script {
    use starcoin_framework::consensus_config;
    use starcoin_framework::epoch;

    // EINVALID_UNCLES_COUNT
    fun adjust_epoch(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let config = consensus_config::get_config();
        let max_uncles_per_block = consensus_config::base_max_uncles_per_block(&config);
        let uncles = max_uncles_per_block + 1;
        let _reward = epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);
    }
}
// check: "Keep(ABORTED { code: 25863"


//# run --signers Genesis
script {
    use starcoin_framework::epoch;
    use starcoin_framework::consensus_config;

    //EUNREACHABLE, block_number > epoch_ref.end_block_number
    fun adjust_epoch(genesis_account: signer) {
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let config = consensus_config::get_config();
        let block_number = 1 + consensus_config::epoch_block_count(&config);
        let _reward = epoch::adjust_epoch(
            &genesis_account, block_number,
            block_time_milliseonds,
            uncles,
            0
        );
    }
}
// check: "Keep(ABORTED { code: 19"


//# run --signers Genesis
script {
    use starcoin_framework::epoch;
    use starcoin_framework::consensus_config;

    //EINVALID_UNCLES_COUNT. If block_number == epoch_ref.end_block_number, uncles should be 0
    fun adjust_epoch(genesis_account: signer) {
        let block_time_milliseonds = 1000;
        let uncles = 1;
        let config = consensus_config::get_config();
        let block_number = consensus_config::epoch_block_count(&config);
        let _reward = epoch::adjust_epoch(
            &genesis_account, block_number,
            block_time_milliseonds,
            uncles,
            0
        );
    }
}
// check: "Keep(ABORTED { code: 25863"


//# run --signers Genesis
script {
    use starcoin_framework::epoch;
    use starcoin_framework::consensus_config;

    //block_number == epoch_ref.end_block_number
    fun adjust_epoch(genesis_account: signer) {
        let block_time_milliseonds = 1000;
        let uncles = 0;
        let config = consensus_config::get_config();
        let block_number = consensus_config::epoch_block_count(&config);
        let _reward = epoch::adjust_epoch(
            &genesis_account,
            block_number,
            block_time_milliseonds,
            uncles,
            0
        );
    }
}
// check: EXECUTED
