//# init -n dev

//# faucet --addr Genesis

//# run --signers Genesis
script {
use StarcoinFramework::ConsensusConfig;
use StarcoinFramework::Epoch;
//use StarcoinFramework::Debug;

    fun adjust_epoch_block_time(genesis_account: signer) {
        let block_number = 1;
        let block_time_milliseonds = 1000;
        let times = 0;
        let config = ConsensusConfig::get_config();
        let base_block_time_target = ConsensusConfig::base_block_time_target(&config);
        let max_block_time_target = ConsensusConfig::max_block_time_target(&config);
        let pre_block_time_target = Epoch::block_time_target();
        while (Epoch::number() < 10) {
            let uncles = 1;
            if (block_number == Epoch::end_block_number()) {
                uncles = 0;
                //Debug::print(&Epoch::block_time_target());
            };
            let _reward = Epoch::adjust_epoch(&genesis_account, block_number, block_time_milliseonds, uncles, 0);

            let block_time_target = Epoch::block_time_target();
            //Debug::print(&block_time_target);
            assert!(pre_block_time_target <= block_time_target, 101);
            assert!(block_time_target >= base_block_time_target, 102);
            assert!(block_time_target <= max_block_time_target, 103);
            times = times + 1;
            block_number = block_number + 1;
            block_time_milliseonds = block_time_milliseonds + block_time_target;
            pre_block_time_target = block_time_target;
        };
    }
}

// check: EXECUTED
