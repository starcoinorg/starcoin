//# init -n dev

//# faucet --addr creator --amount 100000000000

//# block --author=0x2

//# call-api chain.get_block_by_number [1]

//# call-api state.get_with_proof_by_root_raw ["0x1/1/0x1::account::Account","{{$.call-api[0].header.state_root}}"]

//# run --signers creator --args {{$.call-api[0].header.number}}u64  --args  "{{$.call-api[0].header.block_hash}}" --args "{{$.call-api[1]}}"
script{
     use std::vector;
     fun main(_sender: signer, block_number: u64, block_hash: vector<u8>, state_proof: vector<u8>){
         assert!(block_number == 1, 1000);
         assert!(vector::length(&block_hash) == 32, 1001);
         assert!(vector::length(&state_proof) > 32, 1002);
     }
}

//# call-api chain.info

//# block --author=0x3

//# call-api chain.info

//# run --signers creator --args b"{{$.call-api[2].head.block_hash}}" --args "{{$.call-api[3].head.parent_hash}}"
script {
    fun latest(_account: signer, parent_hash: vector<u8>, expect_parent_hash: vector<u8>) {
        assert!(parent_hash == expect_parent_hash, 1003)
    }
}
// check: EXECUTED