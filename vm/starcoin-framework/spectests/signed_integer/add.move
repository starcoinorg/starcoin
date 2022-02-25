//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::SignedInteger64;

fun main() {
    let i1 = SignedInteger64::create_from_raw_value(100, true);
    let zero = SignedInteger64::add_u64(100, copy i1);
    assert!(SignedInteger64::get_value(zero) == 0, 1);

    let negative = SignedInteger64::add_u64(50, copy i1);
    assert!(SignedInteger64::get_value(copy negative) == 50, 2);
    assert!(SignedInteger64::is_negative(copy negative) == true, 3);

    let positive = SignedInteger64::add_u64(150, copy i1);
    assert!(SignedInteger64::get_value(copy positive) == 50, 4);
    assert!(SignedInteger64::is_negative(copy positive) == false, 5);

    let i2 = SignedInteger64::create_from_raw_value(0, false);
    let z2 = SignedInteger64::add_u64(100, copy i2);
    assert!(SignedInteger64::get_value(z2) == 100, 6);
}
}