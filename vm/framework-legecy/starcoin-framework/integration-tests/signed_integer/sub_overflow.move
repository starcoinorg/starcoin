//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::SignedInteger64;

fun main() {
    let i1 = SignedInteger64::create_from_raw_value(18446744073709551615, true);
    let zero = SignedInteger64::sub_u64(1, i1);
    assert!(SignedInteger64::get_value(zero) == 0, 1);
}
}

// check: ARITHMETIC_ERROR