//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::consensus_strategy;

    fun main() {
        assert!(consensus_strategy::get() == 0, 1);
    }
}
