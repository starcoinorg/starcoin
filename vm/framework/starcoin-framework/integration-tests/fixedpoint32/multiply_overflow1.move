//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use std::fixed_point32;

    fun main() {
        let f1 = fixed_point32::create_from_rational(3, 2); // 1.5
        // Multiply the maximum u64 value by 1.5. This should overflow.
        let overflow = fixed_point32::multiply_u64(18446744073709551615, copy f1);
        // The above should fail at runtime so that the following assertion
        // is never even tested.
        assert!(overflow == 999, 1);
    }
}
// check: "Keep(ABORTED { code: 26376"
