//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use starcoin_framework::error;
    fun main() {
        assert!(error::invalid_state(0) == 1, 0);
        assert!(error::requires_address(0) == 2, 1);
        assert!(error::requires_role(0) == 3, 2);
        assert!(error::not_published(0) == 5, 4);
        assert!(error::already_published(0) == 6, 5);
        assert!(error::invalid_argument(0) == 7, 6);
        assert!(error::limit_exceeded(0) == 8, 7);
        assert!(error::internal(0) == 10, 8);
        assert!(error::custom(0) == 255, 9);
    }
}
// check: "Keep(EXECUTED)"