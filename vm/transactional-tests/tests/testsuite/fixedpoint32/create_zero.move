//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
use StarcoinFramework::FixedPoint32;
fun main() {
    let x = FixedPoint32::create_from_rational(0, 1);
    assert!(FixedPoint32::get_raw_value(x) == 0, 0);
}
}
// check: "Keep(EXECUTED)"
