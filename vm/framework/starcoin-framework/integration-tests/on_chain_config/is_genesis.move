//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::Timestamp;

    fun main() {
        assert!(!Timestamp::is_genesis(), 10);
    }
}
