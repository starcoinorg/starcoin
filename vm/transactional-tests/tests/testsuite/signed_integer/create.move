//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::SignedInteger64;

fun main() {
    let i1 = SignedInteger64::create_from_raw_value(4294967296, true);
    let value = SignedInteger64::get_value(copy i1);
    assert!(value == 4294967296, 1);
    let is_negative = SignedInteger64::is_negative(copy i1);
    assert!(is_negative == true, 2);
}
}