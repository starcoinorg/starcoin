//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use std::fixed_point32;

    fun main() {
        let x = fixed_point32::create_from_rational(0, 1);
        assert!(fixed_point32::get_raw_value(x) == 0, 0);
    }
}
// check: "Keep(EXECUTED)"
