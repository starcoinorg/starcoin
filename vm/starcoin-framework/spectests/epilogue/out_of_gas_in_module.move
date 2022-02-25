//# init -n dev

//# faucet --addr default --amount 100000000000000000

//# publish
module default::Swapper {
    use StarcoinFramework::Vector;
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

//# run --signers default --gas-budget 620000
script {
use default::Swapper;
fun main() {
    Swapper::swap_it_up(10000)
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS, location: 0xd98f86e3303c97b00313854b8314f51b::Swapper,"
// check: "gas_used: 620000,"
// check: "Keep(OUT_OF_GAS)"
