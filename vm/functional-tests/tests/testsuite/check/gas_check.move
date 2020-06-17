// Test the gas check flow

//! account: alice, 1000STC

module B {
    struct T {g: u64}

    public fun new(g: u64): T {
        T { g }
    }

    public fun t(this: &T) {
        let g = &this.g;
        *g;
    }
}

//! new-transaction
//! sender: alice

script {
//! gas-price: 1
//! max-gas: 1000
use {{default}}::B;

fun main() {
    let x = B::new(5);
    B::t(&x);
}
}

// check: gas_used
// check: 1000
// check: OUT_OF_GAS

