// Test the gas check flow
//! account: alice, 50000000 0x1::STC::STC

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

//! new-transaction
//! sender: alice
//! gas-price: 1
//! max-gas: 599

script {
    fun main() {
        while (true) {}
    }
}

// check: Discard
// check: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS

//! new-transaction
//! sender: alice
//! gas-price: 1
//! max-gas: 40000001

script {
    fun main() {
        while (true) {}
    }
}

// check: Discard
// check: MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND