script {
use 0x1::FixedPoint32;
fun main() {
    let x = FixedPoint32::create_from_rational(0, 1);
    assert(FixedPoint32::get_raw_value(x) == 0, 0);
}
}
// check: "Keep(EXECUTED)"
