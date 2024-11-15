//# init -n test

//# faucet --addr alice --amount 50000000

//# run --signers alice
script {
    use starcoin_framework::ChainId;

    fun main() {
        assert!(ChainId::get() == 255, 1000);
    }
}
