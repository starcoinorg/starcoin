//# init -n dev

//# faucet --addr creator --amount 100000000000

//# block --author=creator

//# run --signers creator -- 0x1::empty_scripts::empty_script

//# view --address creator --resource 0x1::account::Account

//# run --signers creator --args 2  --args 1
script {
    fun main(_sender: signer, block_number: u64, sequence_number: u64) {
        assert!(block_number == 2, 1000);
        assert!(sequence_number == 1, 1001);
    }
}