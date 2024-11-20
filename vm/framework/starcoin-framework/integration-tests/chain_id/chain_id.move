//# init -n test

//# faucet --addr alice --amount 50000000

//# run --signers alice
script {
    use starcoin_framework::chain_id;

    fun main() {
        assert!(chain_id::get() == 255, 1000);
    }
}
