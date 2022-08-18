//# init --rpc http://halley.seed.starcoin.org  --block-number 1

//# faucet --addr creator --amount 100000000000

//# call-api chain.get_block_by_number [1]

//# run --signers creator --args {{$.call-api[0].header.number}}u64  --args {{$.call-api[0].header.block_hash}}
script{
    use StarcoinFramework::Vector;
    fun main(_sender: signer, block_number: u64, block_hash: vector<u8>){
        assert!(block_number == 1, 1000);
        assert!(Vector::length(&block_hash) == 32, 1001);
    }
}