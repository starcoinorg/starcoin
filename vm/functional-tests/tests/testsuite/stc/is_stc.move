// Check that is_lbr only returns true for STC
script {
use 0x1::STC::{Self, STC};
fun main() {
    assert(STC::is_stc<STC>(), 1);
    assert(!STC::is_stc<bool>(), 3);
}
}
// check: EXECUTED
