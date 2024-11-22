//# init -n dev 

// TODO: this test need the new StateApi `get_state_node_by_node_hash` updated on barnard net.
// //# init --rpc http://barnard.seed.starcoin.org  --block-number 6487000

// //# faucet --addr creator --amount 100000000000

// //# call-api chain.get_block_by_number [6487000]

// //# run --signers creator --args {{$.call-api[0].header.number}}u64  --args {{$.call-api[0].header.block_hash}}
// script{
//     use StarcoinFramework::Vector;
//     fun main(_sender: signer, block_number: u64, block_hash: vector<u8>){
//         assert!(block_number == 6487000, 1000);
//         assert!(vector::length(&block_hash) == 32, 1001);
//         assert!(x"58d3b6aa35ba1f52c809382b876950b6038c4ce9fa078358c0fcf0b072e5ae3d" == block_hash, 1002);
//     }
// }