//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::SignedInteger64;

fun main() {
    let i1 = SignedInteger64::create_from_raw_value(100, true);
    let zero = SignedInteger64::multiply_u64(18446744073709551615, i1);
    assert!(SignedInteger64::get_value(zero) == 9999, 1);
}
}

// check: ARITHMETIC_ERROR