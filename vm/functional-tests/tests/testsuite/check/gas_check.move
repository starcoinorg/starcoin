// Test the gas check flow
//! account: alice, 1000STC

//! sender: alice
//! gas-price: 1
//! max-gas: 1000

script {
fun main() {
    while (true) {}
}
}

// check: gas_used
// check: 1000
// check: OUT_OF_GAS

