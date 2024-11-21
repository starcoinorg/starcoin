//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use std::fixed_point32;

    fun main() {
        // The maximum value is 2^32 - 1. Check that anything larger aborts
        // with an overflow.
        let f1 = fixed_point32::create_from_rational(4294967296, 1); // 2^32
        // The above should fail at runtime so that the following assertion
        // is never even tested.
        assert!(fixed_point32::get_raw_value(f1) == 999, 1);
    }
}
// check: "Keep(ABORTED { code: 26888"
