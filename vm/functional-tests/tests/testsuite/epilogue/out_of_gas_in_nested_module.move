module Swapper {
    use 0x1::Vector;
    public fun call(x: u64) {
        swap_it_up(x)
    }
    public fun swap_it_up(vec_len: u64) {
        let v = Vector::empty();

        let i = 0;
        while (i < vec_len) {
          Vector::push_back(&mut v, i);
          i = i + 1;
        };

        i = 0;

        while (i < vec_len / 2) {
            Vector::swap(&mut v, i, vec_len - i - 1);
            i = i + 1;
        };
    }
}

//! new-transaction
//! max-gas: 621
script {
use {{default}}::Swapper;
fun main() {
    Swapper::call(10000)
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS, location: a4a46d1b1421502568a4a6ac326d7250::Swapper,"
// check: "gas_used: 621,"
// check: "Keep(OUT_OF_GAS)"
