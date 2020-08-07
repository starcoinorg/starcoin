// test adjust epoch full uncle.

//! sender: genesis
script {
use 0x1::Consensus;
use 0x1::Debug;

fun main(genesis_account: &signer) {
   let block_number = 1;
   let block_time = 1;
   let times = 0;
   let init_block_time_target = Consensus::block_time_target();
   let max_block_time_target = Consensus::max_block_time_target();
   while (Consensus::epoch_number() < 10) {
       let uncles = 1;
       if (block_number == Consensus::epoch_end_block_number()) {
           uncles = 0;
           Debug::print(&Consensus::block_time_target());
       };
       let _reward = Consensus::adjust_epoch(genesis_account, block_number, block_time, uncles);

       let block_time_target = Consensus::block_time_target();
       assert(block_time_target >= init_block_time_target, 102);
       assert(block_time_target <= max_block_time_target, 103);
       times = times + 1;
       block_number = block_number + 1;
       block_time = block_time + block_time_target;
   };

}
}

// check: EXECUTED
