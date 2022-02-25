//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::SignedInteger64;

fun main() {
    let i1 = SignedInteger64::create_from_raw_value(100, true);
    let zero = SignedInteger64::divide_u64(0, copy i1);
    assert!(SignedInteger64::get_value(zero) == 0, 1);

    let zero2 = SignedInteger64::divide_u64(1, copy i1);
    assert!(SignedInteger64::get_value(zero2) == 0, 10);

    let negative = SignedInteger64::divide_u64(500, copy i1);
    assert!(SignedInteger64::get_value(copy negative) == 5, 2);
    assert!(SignedInteger64::is_negative(copy negative) == true, 3);

    let i2 = SignedInteger64::create_from_raw_value(100, false);
    let positive = SignedInteger64::divide_u64(200, i2);
    assert!(SignedInteger64::get_value(copy positive) == 2, 4);
    assert!(SignedInteger64::is_negative(copy positive) == false, 5);
}
}