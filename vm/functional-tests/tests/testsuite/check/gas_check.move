// Test the gas check flow
//! account: alice, 1000 0x1::STC::STC

//! sender: alice
//! gas-price: 1
//! max-gas: 1000

script {
fun main() {
    while (true) {}
}
}

// check: OUT_OF_GAS
// check: gas_used
// check: 1000


