//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::SignedInteger64;

fun main() {
    let i1 = SignedInteger64::create_from_raw_value(0, false);
    let zero = SignedInteger64::divide_u64(100, i1);
    assert!(SignedInteger64::get_value(zero) == 0, 1);
}
}

// check: ARITHMETIC_ERROR