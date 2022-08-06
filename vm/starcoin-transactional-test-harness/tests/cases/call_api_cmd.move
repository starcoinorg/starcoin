//# init -n dev

// TODO: this test need the new StateApi `get_state_node_by_node_hash` updated on main net.
// //# init --rpc http://main.seed.starcoin.org  --block-number 6860000 

// //# faucet --addr creator --amount 100000000000

// //# call-api chain.get_block_by_number [6860000]

// //# run --signers creator --args {{$.call-api[0].header.number}}u64  --args {{$.call-api[0].header.block_hash}}
// script{
//     use StarcoinFramework::Vector;
//     fun main(_sender: signer, block_number: u64, block_hash: vector<u8>){
//         assert!(block_number == 6860000, 1000);
//         assert!(Vector::length(&block_hash) == 32, 1001);
//         assert!(x"9b53ef17647e66b8946e2b766f6ec8c3c948c5ff07f7f583890548cd7afba7f5" == block_hash, 1002);
//     }
// }